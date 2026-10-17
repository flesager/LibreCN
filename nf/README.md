# Network functions

One crate per 3GPP network function (AMF, SMF, UPF, NRF, ...), each hosted in
an execution object of `librecn-scheduler`, logging and raising alarms through
`librecn-common`, and registered as a kind in the `librecn` application so the
control EO can deploy it. None yet: the `demo` kind in `app/` stands in.
