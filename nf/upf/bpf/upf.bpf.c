// SPDX-License-Identifier: MIT
/*
 * The UPF's fast path: the kernel carries the sessions it can, the UPF the
 * rest (the fallback, as without it).
 *
 * Uplink (upf_uplink): XDP on the N3 interface. A G-PDU to the UPF's N3
 * address whose TEID is in `uplink`, from the session's UE, on the default
 * QoS flow and within the session's uplink AMBR, loses its outer IPv4, UDP
 * and GTP-U headers and goes on into the kernel's stack, which routes the
 * UE's packet (NAT included): the UPF's socket, its userspace and the TUN
 * device are skipped. Anything else goes on untouched to the UPF's socket:
 * echo, unknown TEIDs, other header layouts, other flows, a spoofed source.
 *
 * Downlink (upf_downlink): TC on the egress of the N6 device (the TUN
 * device the UE addresses are routed to, after NAT). A packet to a UE in
 * `downlink` (a session with its gNB's tunnel), within the session's
 * downlink AMBR, gets Ethernet, IPv4, UDP and GTP-U headers (the default
 * flow) and leaves by the N3 interface, to the gNB's next hop as the
 * kernel's routing and neighbours give it: the TUN device and the UPF are
 * skipped. Anything else goes on to the TUN device: idle UEs (parked, the
 * UE paged), a next hop not yet resolved, too large for N3's MTU, a GSO
 * aggregate, fragments, and packets whose TCP or UDP checksum is still
 * due (the kernel fills it in after this hook: the host's own packets,
 * and those from its virtual interfaces, such as a container's), which
 * the kernel completes for the TUN device.
 *
 * The UPF (nf/upf/src/fast.rs) fills `uplink` and `downlink` from its
 * sessions and reads `counts`. Built by build.sh into upf.bpf.o, which is
 * checked in.
 */
#include <linux/bpf.h>
#include <linux/if_ether.h>
#include <linux/in.h>
#include <linux/ip.h>
#include <linux/pkt_cls.h>
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
/* A downlink G-PDU's GTP-U header, as the UPF writes it: the mandatory 8,
 * the optional 4, one 4-byte PDU Session Container. */
#define G_PDU_HEADER 16
#define ENCAP (GTP_OFFSET + G_PDU_HEADER)

/* The UPF's N3 address (network order), its interface and MTU, set at load. */
volatile const __u32 n3_addr = 0;
volatile const __u32 n3_ifindex = 0;
volatile const __u32 n3_mtu = 1500;

/* A session's uplink, by its TEID (network order). */
struct uplink {
	__u32 ue_ip; /* network order */
	__u32 cell; /* its slot in `counts`; MAX_CELLS: none */
	__u64 rate; /* uplink AMBR, bit/s; 0: no limit */
	__u64 done; /* when the traffic let through is done at the rate (ns) */
};

/* A session's downlink, by its UE's address (network order). */
struct downlink {
	__u32 gnb; /* the gNB's N3 address, network order */
	__u32 teid; /* the gNB's TEID, network order */
	__u32 cell; /* its slot in `counts`; MAX_CELLS: none */
	__u32 pad;
	__u64 rate; /* downlink AMBR, bit/s; 0: no limit */
	__u64 done;
};

/* Per cell slot, and per CPU: what the fast path carried and dropped (the
 * UEs' packets: their bytes without the tunnel's headers). */
struct counts {
	__u64 up_packets;
	__u64 up_bytes;
	__u64 down_packets;
	__u64 down_bytes;
	__u64 rate_limited;
};

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 1024); /* max_sessions, set at load */
	__type(key, __u32);
	__type(value, struct uplink);
} uplink SEC(".maps");

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 1024); /* max_sessions, set at load */
	__type(key, __u32);
	__type(value, struct downlink);
} downlink SEC(".maps");

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
static __always_inline int allow(__u64 rate, __u64 *state, __u64 bytes)
{
	if (!rate)
		return 1;
	__u64 now = bpf_ktime_get_ns();
	__u64 cost = bytes * 8 * 1000000000ULL / rate;
	for (int i = 0; i < 2; i++) {
		__u64 done = *(volatile __u64 *)state;
		if (done > now + BURST_NS)
			return 0;
		__u64 next = (done > now ? done : now) + cost;
		if (__sync_val_compare_and_swap(state, done, next) == done)
			return 1;
	}
	return 1;
}

static __always_inline struct counts *counts_of(__u32 cell)
{
	__u32 slot = cell < MAX_CELLS ? cell : MAX_CELLS;
	return bpf_map_lookup_elem(&counts, &slot);
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

	struct counts *c = counts_of(s->cell);
	if (!c)
		return XDP_PASS;
	if (!allow(s->rate, &s->done, bytes)) {
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
	c->up_packets++;
	c->up_bytes += bytes;
	return XDP_PASS;
}

/* The one's complement sum of an IPv4 header, folded and negated. */
static __always_inline __u16 ip_checksum(const struct iphdr *ip)
{
	const __u16 *w = (const __u16 *)ip;
	__u32 sum = 0;
	for (int i = 0; i < (int)(sizeof(*ip) / 2); i++)
		sum += w[i];
	sum = (sum & 0xffff) + (sum >> 16);
	sum = (sum & 0xffff) + (sum >> 16);
	return ~sum;
}

/*
 * Whether the UE's packet `ip` is a fragment, or TCP or UDP whose checksum
 * is still due (CHECKSUM_PARTIAL): its checksum field holds the pseudo
 * header's sum only. A complete checksum equal to that (once in 65536)
 * only sends the packet to the UPF.
 */
static __always_inline int checksum_due(struct __sk_buff *skb, const struct iphdr *ip)
{
	if (ip->frag_off & bpf_htons(0x3fff))
		return 1;
	__u32 field_at;
	if (ip->protocol == IPPROTO_TCP)
		field_at = 16;
	else if (ip->protocol == IPPROTO_UDP)
		field_at = 6;
	else
		return 0;
	__u32 header = ip->ihl * 4;
	__u16 field;
	if (bpf_skb_load_bytes(skb, header + field_at, &field, sizeof(field)))
		return 1;
	/* In network order throughout: one's complement sums do not mind. */
	__u32 sum = (ip->saddr & 0xffff) + (ip->saddr >> 16) + (ip->daddr & 0xffff) + (ip->daddr >> 16) +
		    bpf_htons(ip->protocol) + bpf_htons(bpf_ntohs(ip->tot_len) - header);
	sum = (sum & 0xffff) + (sum >> 16);
	sum = (sum & 0xffff) + (sum >> 16);
	return (__u16)sum == field;
}

SEC("classifier")
int upf_downlink(struct __sk_buff *skb)
{
	if (skb->protocol != bpf_htons(ETH_P_IP) || skb->gso_segs > 1 || skb->sk)
		return TC_ACT_OK;
	struct iphdr inner;
	if (bpf_skb_load_bytes(skb, 0, &inner, sizeof(inner)) || inner.version != 4 || checksum_due(skb, &inner))
		return TC_ACT_OK;
	struct downlink *s = bpf_map_lookup_elem(&downlink, &inner.daddr);
	if (!s)
		return TC_ACT_OK;
	__u32 len = skb->len;
	if (len + ENCAP - sizeof(struct ethhdr) > n3_mtu)
		return TC_ACT_OK;

	/* The gNB's next hop on N3: the kernel's route and neighbour. */
	struct bpf_fib_lookup fib = {};
	fib.family = 2; /* AF_INET */
	fib.ipv4_src = n3_addr;
	fib.ipv4_dst = s->gnb;
	fib.ifindex = n3_ifindex;
	if (bpf_fib_lookup(skb, &fib, sizeof(fib), BPF_FIB_LOOKUP_OUTPUT) != BPF_FIB_LKUP_RET_SUCCESS)
		return TC_ACT_OK;

	struct counts *c = counts_of(s->cell);
	if (!c)
		return TC_ACT_OK;
	if (!allow(s->rate, &s->done, len)) {
		c->rate_limited++;
		return TC_ACT_SHOT;
	}

	struct {
		struct ethhdr eth;
		struct iphdr ip;
		struct udphdr udp;
		__u8 gtp[G_PDU_HEADER];
	} __attribute__((packed)) h = {};
	__builtin_memcpy(h.eth.h_dest, fib.dmac, ETH_ALEN);
	__builtin_memcpy(h.eth.h_source, fib.smac, ETH_ALEN);
	h.eth.h_proto = bpf_htons(ETH_P_IP);
	struct iphdr ip = {};
	ip.version = 4;
	ip.ihl = 5;
	ip.tot_len = bpf_htons(len + ENCAP - sizeof(struct ethhdr));
	ip.frag_off = bpf_htons(0x4000); /* don't fragment: it fits */
	ip.ttl = 64;
	ip.protocol = IPPROTO_UDP;
	ip.saddr = n3_addr;
	ip.daddr = s->gnb;
	ip.check = ip_checksum(&ip);
	__builtin_memcpy(&h.ip, &ip, sizeof(ip));
	h.udp.source = bpf_htons(GTPU_PORT);
	h.udp.dest = bpf_htons(GTPU_PORT);
	h.udp.len = bpf_htons(len + sizeof(struct udphdr) + G_PDU_HEADER);
	/* E flag, G-PDU, the length after the mandatory 8 bytes, the TEID; then
	 * DL PDU SESSION INFORMATION: PDU type 0, the QFI. */
	__u16 gtp_len = bpf_htons(len + G_PDU_HEADER - 8);
	h.gtp[0] = GTP_FLAGS | GTP_E;
	h.gtp[1] = G_PDU;
	__builtin_memcpy(&h.gtp[2], &gtp_len, 2);
	__builtin_memcpy(&h.gtp[4], &s->teid, 4);
	h.gtp[11] = PDU_SESSION_CONTAINER;
	h.gtp[12] = 1;
	h.gtp[14] = DEFAULT_QFI;

	if (bpf_skb_change_head(skb, ENCAP, 0) || bpf_skb_store_bytes(skb, 0, &h, sizeof(h), 0))
		return TC_ACT_SHOT;
	c->down_packets++;
	c->down_bytes += len;
	return bpf_redirect(fib.ifindex, 0);
}

char LICENSE[] SEC("license") = "Dual MIT/GPL";
