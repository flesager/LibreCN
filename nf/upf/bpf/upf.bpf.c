// SPDX-License-Identifier: MIT
/*
 * The UPF's fast path, uplink: XDP on the N3 interface. A G-PDU to the
 * UPF's N3 address whose TEID is in `uplink`, from the session's UE, on
 * the default QoS flow and within the session's uplink AMBR, loses its
 * outer IPv4, UDP and GTP-U headers and goes on into the kernel's stack,
 * which routes the UE's packet (NAT included): the UPF's socket, its
 * userspace and the TUN device are skipped. Anything else goes on
 * untouched to the UPF's socket, which handles it as without the fast
 * path (the fallback): echo, unknown TEIDs, other header layouts, other
 * flows, a spoofed source.
 *
 * The UPF (nf/upf/src/fast.rs) fills `uplink` from its sessions and
 * reads `counts`. Built by build.sh into upf.bpf.o, which is checked in.
 */
#include <linux/bpf.h>
#include <linux/if_ether.h>
#include <linux/in.h>
#include <linux/ip.h>
#include <linux/udp.h>
#include <bpf/bpf_endian.h>
#include <bpf/bpf_helpers.h>

#define GTPU_PORT 2152
/* nf/upf/src/lib.rs: MAX_CELLS; qos.rs: BURST; the policy's DEFAULT_QFI. */
#define MAX_CELLS 1024
#define BURST_NS 100000000ULL
#define DEFAULT_QFI 1

/* GTP-U (TS 29.281): version 1, protocol type GTP; the optional fields. */
#define GTP_FLAGS 0x30
#define GTP_E 0x04
#define GTP_S 0x02
#define GTP_PN 0x01
#define G_PDU 0xff
#define PDU_SESSION_CONTAINER 0x85
/* Ethernet, IPv4 without options, UDP: where GTP-U starts. */
#define GTP_OFFSET (sizeof(struct ethhdr) + sizeof(struct iphdr) + sizeof(struct udphdr))

/* The UPF's N3 address (network order), set at load. */
volatile const __u32 n3_addr = 0;

/* A session's uplink, by its TEID (network order). */
struct uplink {
	__u32 ue_ip; /* network order */
	__u32 cell; /* its slot in `counts`; MAX_CELLS: none */
	__u64 rate; /* uplink AMBR, bit/s; 0: no limit */
	__u64 done; /* when the traffic let through is done at the rate (ns) */
};

/* Per cell slot, and per CPU: what the fast path carried and dropped. */
struct counts {
	__u64 packets;
	__u64 bytes;
	__u64 rate_limited;
};

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 1024); /* max_sessions, set at load */
	__type(key, __u32);
	__type(value, struct uplink);
} uplink SEC(".maps");

struct {
	__uint(type, BPF_MAP_TYPE_PERCPU_ARRAY);
	__uint(max_entries, MAX_CELLS + 1);
	__type(key, __u32);
	__type(value, struct counts);
} counts SEC(".maps");

/*
 * The session's rate limit, a GCRA as in qos.rs: a packet conforms unless
 * the traffic let through would take more than BURST_NS beyond now; if it
 * does, its share is taken. Another CPU may take a share at the same time:
 * compare and swap, once more if it lost, then let the packet through.
 */
static __always_inline int allow(struct uplink *s, __u64 bytes)
{
	if (!s->rate)
		return 1;
	__u64 now = bpf_ktime_get_ns();
	__u64 cost = bytes * 8 * 1000000000ULL / s->rate;
	for (int i = 0; i < 2; i++) {
		__u64 done = *(volatile __u64 *)&s->done;
		if (done > now + BURST_NS)
			return 0;
		__u64 next = (done > now ? done : now) + cost;
		if (__sync_val_compare_and_swap(&s->done, done, next) == done)
			return 1;
	}
	return 1;
}

SEC("xdp")
int upf_uplink(struct xdp_md *ctx)
{
	__u32 size = ctx->data_end - ctx->data;
	void *data = (void *)(long)ctx->data;
	void *end = (void *)(long)ctx->data_end;

	struct ethhdr *eth = data;
	if ((void *)(eth + 1) > end || eth->h_proto != bpf_htons(ETH_P_IP))
		return XDP_PASS;
	struct iphdr *ip = (void *)(eth + 1);
	if ((void *)(ip + 1) > end || ip->ihl != 5 || ip->protocol != IPPROTO_UDP || ip->daddr != n3_addr ||
	    (ip->frag_off & bpf_htons(0x3fff)))
		return XDP_PASS;
	struct udphdr *udp = (void *)(ip + 1);
	__u8 *gtp = (void *)(udp + 1);
	if ((void *)(gtp + 16) > end || udp->dest != bpf_htons(GTPU_PORT) || gtp[1] != G_PDU)
		return XDP_PASS;

	/* The layouts taken: no optional field; or the optional ones, with
	 * no extension header or one PDU Session Container of 4 bytes, the
	 * last, on the default flow. */
	__u8 flags = gtp[0];
	__u32 hdr;
	if (flags == GTP_FLAGS) {
		hdr = 8;
	} else if ((flags & ~(GTP_S | GTP_PN)) == GTP_FLAGS) {
		hdr = 12;
	} else if ((flags & ~(GTP_S | GTP_PN)) == (GTP_FLAGS | GTP_E) && gtp[11] == PDU_SESSION_CONTAINER &&
		   gtp[12] == 1 && (gtp[14] & 0x3f) == DEFAULT_QFI && gtp[15] == 0) {
		hdr = 16;
	} else {
		return XDP_PASS;
	}
	/* The length field counts what follows the mandatory 8 bytes. */
	__u32 len = (__u32)gtp[2] << 8 | gtp[3];
	if (GTP_OFFSET + 8 + len > size || len + 8 < hdr + sizeof(struct iphdr))
		return XDP_PASS;
	__u64 bytes = len + 8 - hdr;

	__u32 teid;
	__builtin_memcpy(&teid, gtp + 4, sizeof(teid));
	struct uplink *s = bpf_map_lookup_elem(&uplink, &teid);
	if (!s)
		return XDP_PASS;
	struct iphdr *inner = (void *)(gtp + hdr);
	if ((void *)(inner + 1) > end || inner->version != 4 || inner->saddr != s->ue_ip)
		return XDP_PASS;

	__u32 cell = s->cell < MAX_CELLS ? s->cell : MAX_CELLS;
	struct counts *c = bpf_map_lookup_elem(&counts, &cell);
	if (!c)
		return XDP_PASS;
	if (!allow(s, bytes)) {
		c->rate_limited++;
		return XDP_DROP;
	}

	/* Decapsulated: the Ethernet header moves up to the UE's packet. */
	struct ethhdr copy = *eth;
	if (bpf_xdp_adjust_head(ctx, (int)(sizeof(struct iphdr) + sizeof(struct udphdr) + hdr)))
		return XDP_PASS;
	data = (void *)(long)ctx->data;
	end = (void *)(long)ctx->data_end;
	eth = data;
	if ((void *)(eth + 1) > end)
		return XDP_DROP;
	__builtin_memcpy(eth, &copy, sizeof(copy));
	c->packets++;
	c->bytes += bytes;
	return XDP_PASS;
}

char LICENSE[] SEC("license") = "Dual MIT/GPL";
