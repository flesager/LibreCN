#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, unused, clippy::too_many_arguments)]
pub mod ngap_common_data_types {
    extern crate alloc;
    use core::borrow::Borrow;
    use rasn::prelude::*;
    use std::sync::LazyLock;
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum Criticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum Presence {
        optional = 0,
        conditional = 1,
        mandatory = 2,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags, identifier = "PrivateIE-ID")]
    pub enum PrivateIEID {
        #[rasn(value("0..=65535"))]
        local(u16),
        global(ObjectIdentifier),
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=255"))]
    pub struct ProcedureCode(pub u8);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=65535"))]
    pub struct ProtocolExtensionID(pub u16);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "ProtocolIE-ID", value("0..=65535"))]
    pub struct ProtocolIEID(pub u16);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum TriggeringMessage {
        #[rasn(identifier = "initiating-message")]
        initiating_message = 0,
        #[rasn(identifier = "successful-outcome")]
        successful_outcome = 1,
        #[rasn(identifier = "unsuccessfull-outcome")]
        unsuccessfull_outcome = 2,
    }
}
#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, unused, clippy::too_many_arguments)]
pub mod ngap_constants {
    extern crate alloc;
    use super::ngap_common_data_types::*;
    use core::borrow::Borrow;
    use rasn::prelude::*;
    use std::sync::LazyLock;
    pub const ID_AMF_TNLASSOCIATION_FAILED_TO_SETUP_LIST: ProtocolIEID = ProtocolIEID(4);
    pub const ID_AMF_TNLASSOCIATION_SETUP_LIST: ProtocolIEID = ProtocolIEID(5);
    pub const ID_AMF_TNLASSOCIATION_TO_ADD_LIST: ProtocolIEID = ProtocolIEID(6);
    pub const ID_AMF_TNLASSOCIATION_TO_REMOVE_LIST: ProtocolIEID = ProtocolIEID(7);
    pub const ID_AMF_TNLASSOCIATION_TO_UPDATE_LIST: ProtocolIEID = ProtocolIEID(8);
    pub const ID_AMF_UE_NGAP_ID: ProtocolIEID = ProtocolIEID(10);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Elementary Procedures"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    pub const ID_AMFCONFIGURATION_UPDATE: ProcedureCode = ProcedureCode(0);
    pub const ID_AMFNAME: ProtocolIEID = ProtocolIEID(1);
    pub const ID_AMFOVERLOAD_RESPONSE: ProtocolIEID = ProtocolIEID(2);
    pub const ID_AMFSET_ID: ProtocolIEID = ProtocolIEID(3);
    pub const ID_AMFSTATUS_INDICATION: ProcedureCode = ProcedureCode(1);
    pub const ID_AMFTRAFFIC_LOAD_REDUCTION_INDICATION: ProtocolIEID = ProtocolIEID(9);
    pub const ID_ADDITIONAL_DLFORWARDING_UPTNLINFORMATION: ProtocolIEID = ProtocolIEID(152);
    pub const ID_ADDITIONAL_DLQOS_FLOW_PER_TNLINFORMATION: ProtocolIEID = ProtocolIEID(155);
    pub const ID_ADDITIONAL_DLUPTNLINFORMATION_FOR_HOLIST: ProtocolIEID = ProtocolIEID(153);
    pub const ID_ADDITIONAL_NGU_UP_TNLINFORMATION: ProtocolIEID = ProtocolIEID(154);
    pub const ID_ADDITIONAL_UL_NGU_UP_TNLINFORMATION: ProtocolIEID = ProtocolIEID(126);
    pub const ID_ADDITIONAL_ULFORWARDING_UPTNLINFORMATION: ProtocolIEID = ProtocolIEID(172);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " IEs"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    pub const ID_ALLOWED_NSSAI: ProtocolIEID = ProtocolIEID(0);
    pub const ID_ASSISTANCE_DATA_FOR_PAGING: ProtocolIEID = ProtocolIEID(11);
    pub const ID_BROADCAST_CANCELLED_AREA_LIST: ProtocolIEID = ProtocolIEID(12);
    pub const ID_BROADCAST_COMPLETED_AREA_LIST: ProtocolIEID = ProtocolIEID(13);
    pub const ID_CNASSISTED_RANTUNING: ProtocolIEID = ProtocolIEID(165);
    pub const ID_CNTYPE_RESTRICTIONS_FOR_EQUIVALENT: ProtocolIEID = ProtocolIEID(160);
    pub const ID_CNTYPE_RESTRICTIONS_FOR_SERVING: ProtocolIEID = ProtocolIEID(161);
    pub const ID_CANCEL_ALL_WARNING_MESSAGES: ProtocolIEID = ProtocolIEID(14);
    pub const ID_CAUSE: ProtocolIEID = ProtocolIEID(15);
    pub const ID_CELL_IDLIST_FOR_RESTART: ProtocolIEID = ProtocolIEID(16);
    pub const ID_CELL_TRAFFIC_TRACE: ProcedureCode = ProcedureCode(2);
    pub const ID_COMMON_NETWORK_INSTANCE: ProtocolIEID = ProtocolIEID(166);
    pub const ID_CONCURRENT_WARNING_MESSAGE_IND: ProtocolIEID = ProtocolIEID(17);
    pub const ID_CORE_NETWORK_ASSISTANCE_INFORMATION_FOR_INACTIVE: ProtocolIEID = ProtocolIEID(18);
    pub const ID_CRITICALITY_DIAGNOSTICS: ProtocolIEID = ProtocolIEID(19);
    pub const ID_DL_NGU_UP_TNLINFORMATION: ProtocolIEID = ProtocolIEID(128);
    pub const ID_DATA_CODING_SCHEME: ProtocolIEID = ProtocolIEID(20);
    pub const ID_DATA_FORWARDING_NOT_POSSIBLE: ProtocolIEID = ProtocolIEID(127);
    pub const ID_DEACTIVATE_TRACE: ProcedureCode = ProcedureCode(3);
    pub const ID_DEFAULT_PAGING_DRX: ProtocolIEID = ProtocolIEID(21);
    pub const ID_DIRECT_FORWARDING_PATH_AVAILABILITY: ProtocolIEID = ProtocolIEID(22);
    pub const ID_DOWNLINK_NASTRANSPORT: ProcedureCode = ProcedureCode(4);
    pub const ID_DOWNLINK_NON_UEASSOCIATED_NRPPA_TRANSPORT: ProcedureCode = ProcedureCode(5);
    pub const ID_DOWNLINK_RANCONFIGURATION_TRANSFER: ProcedureCode = ProcedureCode(6);
    pub const ID_DOWNLINK_RANSTATUS_TRANSFER: ProcedureCode = ProcedureCode(7);
    pub const ID_DOWNLINK_UEASSOCIATED_NRPPA_TRANSPORT: ProcedureCode = ProcedureCode(8);
    pub const ID_ENDC_SONCONFIGURATION_TRANSFER_DL: ProtocolIEID = ProtocolIEID(157);
    pub const ID_ENDC_SONCONFIGURATION_TRANSFER_UL: ProtocolIEID = ProtocolIEID(158);
    pub const ID_EUTRA_CGI: ProtocolIEID = ProtocolIEID(25);
    pub const ID_EMERGENCY_AREA_IDLIST_FOR_RESTART: ProtocolIEID = ProtocolIEID(23);
    pub const ID_EMERGENCY_FALLBACK_INDICATOR: ProtocolIEID = ProtocolIEID(24);
    pub const ID_ENDPOINT_IPADDRESS_AND_PORT: ProtocolIEID = ProtocolIEID(169);
    pub const ID_ERROR_INDICATION: ProcedureCode = ProcedureCode(9);
    pub const ID_FIVE_G_S_TMSI: ProtocolIEID = ProtocolIEID(26);
    pub const ID_GUAMI: ProtocolIEID = ProtocolIEID(28);
    pub const ID_GLOBAL_RANNODE_ID: ProtocolIEID = ProtocolIEID(27);
    pub const ID_HANDOVER_CANCEL: ProcedureCode = ProcedureCode(10);
    pub const ID_HANDOVER_FLAG: ProtocolIEID = ProtocolIEID(143);
    pub const ID_HANDOVER_NOTIFICATION: ProcedureCode = ProcedureCode(11);
    pub const ID_HANDOVER_PREPARATION: ProcedureCode = ProcedureCode(12);
    pub const ID_HANDOVER_RESOURCE_ALLOCATION: ProcedureCode = ProcedureCode(13);
    pub const ID_HANDOVER_TYPE: ProtocolIEID = ProtocolIEID(29);
    pub const ID_IMSVOICE_SUPPORT_INDICATOR: ProtocolIEID = ProtocolIEID(30);
    pub const ID_INDEX_TO_RFSP: ProtocolIEID = ProtocolIEID(31);
    pub const ID_INFO_ON_RECOMMENDED_CELLS_AND_RANNODES_FOR_PAGING: ProtocolIEID = ProtocolIEID(32);
    pub const ID_INITIAL_CONTEXT_SETUP: ProcedureCode = ProcedureCode(14);
    pub const ID_INITIAL_UEMESSAGE: ProcedureCode = ProcedureCode(15);
    pub const ID_LAST_EUTRAN_PLMNIDENTITY: ProtocolIEID = ProtocolIEID(150);
    pub const ID_LOCATION_REPORT: ProcedureCode = ProcedureCode(18);
    pub const ID_LOCATION_REPORTING_ADDITIONAL_INFO: ProtocolIEID = ProtocolIEID(170);
    pub const ID_LOCATION_REPORTING_CONTROL: ProcedureCode = ProcedureCode(16);
    pub const ID_LOCATION_REPORTING_FAILURE_INDICATION: ProcedureCode = ProcedureCode(17);
    pub const ID_LOCATION_REPORTING_REQUEST_TYPE: ProtocolIEID = ProtocolIEID(33);
    pub const ID_MASKED_IMEISV: ProtocolIEID = ProtocolIEID(34);
    pub const ID_MAXIMUM_INTEGRITY_PROTECTED_DATA_RATE_DL: ProtocolIEID = ProtocolIEID(151);
    pub const ID_MESSAGE_IDENTIFIER: ProtocolIEID = ProtocolIEID(35);
    pub const ID_MOBILITY_RESTRICTION_LIST: ProtocolIEID = ProtocolIEID(36);
    pub const ID_NAS_PDU: ProtocolIEID = ProtocolIEID(38);
    pub const ID_NASC: ProtocolIEID = ProtocolIEID(37);
    pub const ID_NASNON_DELIVERY_INDICATION: ProcedureCode = ProcedureCode(19);
    pub const ID_NASSECURITY_PARAMETERS_FROM_NGRAN: ProtocolIEID = ProtocolIEID(39);
    pub const ID_NGAP_MESSAGE: ProtocolIEID = ProtocolIEID(42);
    pub const ID_NGRAN_CGI: ProtocolIEID = ProtocolIEID(43);
    pub const ID_NGRAN_TNLASSOCIATION_TO_REMOVE_LIST: ProtocolIEID = ProtocolIEID(167);
    pub const ID_NGRANTRACE_ID: ProtocolIEID = ProtocolIEID(44);
    pub const ID_NGRESET: ProcedureCode = ProcedureCode(20);
    pub const ID_NGSETUP: ProcedureCode = ProcedureCode(21);
    pub const ID_NR_CGI: ProtocolIEID = ProtocolIEID(45);
    pub const ID_NRPPA_PDU: ProtocolIEID = ProtocolIEID(46);
    pub const ID_NETWORK_INSTANCE: ProtocolIEID = ProtocolIEID(129);
    pub const ID_NEW_AMF_UE_NGAP_ID: ProtocolIEID = ProtocolIEID(40);
    pub const ID_NEW_GUAMI: ProtocolIEID = ProtocolIEID(162);
    pub const ID_NEW_SECURITY_CONTEXT_IND: ProtocolIEID = ProtocolIEID(41);
    pub const ID_NUMBER_OF_BROADCASTS_REQUESTED: ProtocolIEID = ProtocolIEID(47);
    pub const ID_OLD_AMF: ProtocolIEID = ProtocolIEID(48);
    pub const ID_OLD_ASSOCIATED_QOS_FLOW_LIST_ULENDMARKEREXPECTED: ProtocolIEID = ProtocolIEID(159);
    pub const ID_OVERLOAD_START: ProcedureCode = ProcedureCode(22);
    pub const ID_OVERLOAD_START_NSSAILIST: ProtocolIEID = ProtocolIEID(49);
    pub const ID_OVERLOAD_STOP: ProcedureCode = ProcedureCode(23);
    pub const ID_PDUSESSION_AGGREGATE_MAXIMUM_BIT_RATE: ProtocolIEID = ProtocolIEID(130);
    pub const ID_PDUSESSION_RESOURCE_ADMITTED_LIST: ProtocolIEID = ProtocolIEID(53);
    pub const ID_PDUSESSION_RESOURCE_FAILED_TO_MODIFY_LIST_MOD_CFM: ProtocolIEID = ProtocolIEID(131);
    pub const ID_PDUSESSION_RESOURCE_FAILED_TO_MODIFY_LIST_MOD_RES: ProtocolIEID = ProtocolIEID(54);
    pub const ID_PDUSESSION_RESOURCE_FAILED_TO_SETUP_LIST_CXT_FAIL: ProtocolIEID = ProtocolIEID(132);
    pub const ID_PDUSESSION_RESOURCE_FAILED_TO_SETUP_LIST_CXT_RES: ProtocolIEID = ProtocolIEID(55);
    pub const ID_PDUSESSION_RESOURCE_FAILED_TO_SETUP_LIST_HOACK: ProtocolIEID = ProtocolIEID(56);
    pub const ID_PDUSESSION_RESOURCE_FAILED_TO_SETUP_LIST_PSREQ: ProtocolIEID = ProtocolIEID(57);
    pub const ID_PDUSESSION_RESOURCE_FAILED_TO_SETUP_LIST_SURES: ProtocolIEID = ProtocolIEID(58);
    pub const ID_PDUSESSION_RESOURCE_HANDOVER_LIST: ProtocolIEID = ProtocolIEID(59);
    pub const ID_PDUSESSION_RESOURCE_LIST_CXT_REL_CPL: ProtocolIEID = ProtocolIEID(60);
    pub const ID_PDUSESSION_RESOURCE_LIST_CXT_REL_REQ: ProtocolIEID = ProtocolIEID(133);
    pub const ID_PDUSESSION_RESOURCE_LIST_HORQD: ProtocolIEID = ProtocolIEID(61);
    pub const ID_PDUSESSION_RESOURCE_MODIFY: ProcedureCode = ProcedureCode(26);
    pub const ID_PDUSESSION_RESOURCE_MODIFY_INDICATION: ProcedureCode = ProcedureCode(27);
    pub const ID_PDUSESSION_RESOURCE_MODIFY_LIST_MOD_CFM: ProtocolIEID = ProtocolIEID(62);
    pub const ID_PDUSESSION_RESOURCE_MODIFY_LIST_MOD_IND: ProtocolIEID = ProtocolIEID(63);
    pub const ID_PDUSESSION_RESOURCE_MODIFY_LIST_MOD_REQ: ProtocolIEID = ProtocolIEID(64);
    pub const ID_PDUSESSION_RESOURCE_MODIFY_LIST_MOD_RES: ProtocolIEID = ProtocolIEID(65);
    pub const ID_PDUSESSION_RESOURCE_NOTIFY: ProcedureCode = ProcedureCode(30);
    pub const ID_PDUSESSION_RESOURCE_NOTIFY_LIST: ProtocolIEID = ProtocolIEID(66);
    pub const ID_PDUSESSION_RESOURCE_RELEASE: ProcedureCode = ProcedureCode(28);
    pub const ID_PDUSESSION_RESOURCE_RELEASE_RESPONSE_TRANSFER: ProtocolIEID = ProtocolIEID(145);
    pub const ID_PDUSESSION_RESOURCE_RELEASED_LIST_NOT: ProtocolIEID = ProtocolIEID(67);
    pub const ID_PDUSESSION_RESOURCE_RELEASED_LIST_PSACK: ProtocolIEID = ProtocolIEID(68);
    pub const ID_PDUSESSION_RESOURCE_RELEASED_LIST_PSFAIL: ProtocolIEID = ProtocolIEID(69);
    pub const ID_PDUSESSION_RESOURCE_RELEASED_LIST_REL_RES: ProtocolIEID = ProtocolIEID(70);
    pub const ID_PDUSESSION_RESOURCE_SECONDARY_RATUSAGE_LIST: ProtocolIEID = ProtocolIEID(142);
    pub const ID_PDUSESSION_RESOURCE_SETUP: ProcedureCode = ProcedureCode(29);
    pub const ID_PDUSESSION_RESOURCE_SETUP_LIST_CXT_REQ: ProtocolIEID = ProtocolIEID(71);
    pub const ID_PDUSESSION_RESOURCE_SETUP_LIST_CXT_RES: ProtocolIEID = ProtocolIEID(72);
    pub const ID_PDUSESSION_RESOURCE_SETUP_LIST_HOREQ: ProtocolIEID = ProtocolIEID(73);
    pub const ID_PDUSESSION_RESOURCE_SETUP_LIST_SUREQ: ProtocolIEID = ProtocolIEID(74);
    pub const ID_PDUSESSION_RESOURCE_SETUP_LIST_SURES: ProtocolIEID = ProtocolIEID(75);
    pub const ID_PDUSESSION_RESOURCE_SWITCHED_LIST: ProtocolIEID = ProtocolIEID(77);
    pub const ID_PDUSESSION_RESOURCE_TO_BE_SWITCHED_DLLIST: ProtocolIEID = ProtocolIEID(76);
    pub const ID_PDUSESSION_RESOURCE_TO_RELEASE_LIST_HOCMD: ProtocolIEID = ProtocolIEID(78);
    pub const ID_PDUSESSION_RESOURCE_TO_RELEASE_LIST_REL_CMD: ProtocolIEID = ProtocolIEID(79);
    pub const ID_PDUSESSION_TYPE: ProtocolIEID = ProtocolIEID(134);
    pub const ID_PLMNSUPPORT_LIST: ProtocolIEID = ProtocolIEID(80);
    pub const ID_PSCELL_INFORMATION: ProtocolIEID = ProtocolIEID(149);
    pub const ID_PWSCANCEL: ProcedureCode = ProcedureCode(32);
    pub const ID_PWSFAILED_CELL_IDLIST: ProtocolIEID = ProtocolIEID(81);
    pub const ID_PWSFAILURE_INDICATION: ProcedureCode = ProcedureCode(33);
    pub const ID_PWSRESTART_INDICATION: ProcedureCode = ProcedureCode(34);
    pub const ID_PAGING: ProcedureCode = ProcedureCode(24);
    pub const ID_PAGING_DRX: ProtocolIEID = ProtocolIEID(50);
    pub const ID_PAGING_ORIGIN: ProtocolIEID = ProtocolIEID(51);
    pub const ID_PAGING_PRIORITY: ProtocolIEID = ProtocolIEID(52);
    pub const ID_PATH_SWITCH_REQUEST: ProcedureCode = ProcedureCode(25);
    pub const ID_PRIVATE_MESSAGE: ProcedureCode = ProcedureCode(31);
    pub const ID_QOS_FLOW_ADD_OR_MODIFY_REQUEST_LIST: ProtocolIEID = ProtocolIEID(135);
    pub const ID_QOS_FLOW_SETUP_REQUEST_LIST: ProtocolIEID = ProtocolIEID(136);
    pub const ID_QOS_FLOW_TO_RELEASE_LIST: ProtocolIEID = ProtocolIEID(137);
    pub const ID_RAN_UE_NGAP_ID: ProtocolIEID = ProtocolIEID(85);
    pub const ID_RANCONFIGURATION_UPDATE: ProcedureCode = ProcedureCode(35);
    pub const ID_RANNODE_NAME: ProtocolIEID = ProtocolIEID(82);
    pub const ID_RANPAGING_PRIORITY: ProtocolIEID = ProtocolIEID(83);
    pub const ID_RANSTATUS_TRANSFER_TRANSPARENT_CONTAINER: ProtocolIEID = ProtocolIEID(84);
    pub const ID_RRCESTABLISHMENT_CAUSE: ProtocolIEID = ProtocolIEID(90);
    pub const ID_RRCINACTIVE_TRANSITION_REPORT: ProcedureCode = ProcedureCode(37);
    pub const ID_RRCINACTIVE_TRANSITION_REPORT_REQUEST: ProtocolIEID = ProtocolIEID(91);
    pub const ID_RRCSTATE: ProtocolIEID = ProtocolIEID(92);
    pub const ID_REDIRECTION_VOICE_FALLBACK: ProtocolIEID = ProtocolIEID(146);
    pub const ID_RELATIVE_AMFCAPACITY: ProtocolIEID = ProtocolIEID(86);
    pub const ID_REPETITION_PERIOD: ProtocolIEID = ProtocolIEID(87);
    pub const ID_REROUTE_NASREQUEST: ProcedureCode = ProcedureCode(36);
    pub const ID_RESET_TYPE: ProtocolIEID = ProtocolIEID(88);
    pub const ID_ROUTING_ID: ProtocolIEID = ProtocolIEID(89);
    pub const ID_S_NSSAI: ProtocolIEID = ProtocolIEID(148);
    pub const ID_SCTP_TLAS: ProtocolIEID = ProtocolIEID(173);
    pub const ID_SONCONFIGURATION_TRANSFER_DL: ProtocolIEID = ProtocolIEID(98);
    pub const ID_SONCONFIGURATION_TRANSFER_UL: ProtocolIEID = ProtocolIEID(99);
    pub const ID_SECONDARY_RATDATA_USAGE_REPORT: ProcedureCode = ProcedureCode(52);
    pub const ID_SECONDARY_RATUSAGE_INFORMATION: ProtocolIEID = ProtocolIEID(144);
    pub const ID_SECURITY_CONTEXT: ProtocolIEID = ProtocolIEID(93);
    pub const ID_SECURITY_INDICATION: ProtocolIEID = ProtocolIEID(138);
    pub const ID_SECURITY_KEY: ProtocolIEID = ProtocolIEID(94);
    pub const ID_SECURITY_RESULT: ProtocolIEID = ProtocolIEID(156);
    pub const ID_SELECTED_PLMNIDENTITY: ProtocolIEID = ProtocolIEID(174);
    pub const ID_SERIAL_NUMBER: ProtocolIEID = ProtocolIEID(95);
    pub const ID_SERVED_GUAMILIST: ProtocolIEID = ProtocolIEID(96);
    pub const ID_SLICE_SUPPORT_LIST: ProtocolIEID = ProtocolIEID(97);
    pub const ID_SOURCE_AMF_UE_NGAP_ID: ProtocolIEID = ProtocolIEID(100);
    pub const ID_SOURCE_TO_TARGET_AMFINFORMATION_REROUTE: ProtocolIEID = ProtocolIEID(171);
    pub const ID_SOURCE_TO_TARGET_TRANSPARENT_CONTAINER: ProtocolIEID = ProtocolIEID(101);
    pub const ID_SUPPORTED_TALIST: ProtocolIEID = ProtocolIEID(102);
    pub const ID_TAILIST_FOR_PAGING: ProtocolIEID = ProtocolIEID(103);
    pub const ID_TAILIST_FOR_RESTART: ProtocolIEID = ProtocolIEID(104);
    pub const ID_TNLASSOCIATION_TRANSPORT_LAYER_ADDRESS_NGRAN: ProtocolIEID = ProtocolIEID(168);
    pub const ID_TARGET_ID: ProtocolIEID = ProtocolIEID(105);
    pub const ID_TARGET_TO_SOURCE_TRANSPARENT_CONTAINER: ProtocolIEID = ProtocolIEID(106);
    pub const ID_TIME_TO_WAIT: ProtocolIEID = ProtocolIEID(107);
    pub const ID_TRACE_ACTIVATION: ProtocolIEID = ProtocolIEID(108);
    pub const ID_TRACE_COLLECTION_ENTITY_IPADDRESS: ProtocolIEID = ProtocolIEID(109);
    pub const ID_TRACE_FAILURE_INDICATION: ProcedureCode = ProcedureCode(38);
    pub const ID_TRACE_START: ProcedureCode = ProcedureCode(39);
    pub const ID_UE_NGAP_IDS: ProtocolIEID = ProtocolIEID(114);
    pub const ID_UE_ASSOCIATED_LOGICAL_NG_CONNECTION_LIST: ProtocolIEID = ProtocolIEID(111);
    pub const ID_UEAGGREGATE_MAXIMUM_BIT_RATE: ProtocolIEID = ProtocolIEID(110);
    pub const ID_UECONTEXT_MODIFICATION: ProcedureCode = ProcedureCode(40);
    pub const ID_UECONTEXT_RELEASE: ProcedureCode = ProcedureCode(41);
    pub const ID_UECONTEXT_RELEASE_REQUEST: ProcedureCode = ProcedureCode(42);
    pub const ID_UECONTEXT_REQUEST: ProtocolIEID = ProtocolIEID(112);
    pub const ID_UEPAGING_IDENTITY: ProtocolIEID = ProtocolIEID(115);
    pub const ID_UEPRESENCE_IN_AREA_OF_INTEREST_LIST: ProtocolIEID = ProtocolIEID(116);
    pub const ID_UERADIO_CAPABILITY: ProtocolIEID = ProtocolIEID(117);
    pub const ID_UERADIO_CAPABILITY_CHECK: ProcedureCode = ProcedureCode(43);
    pub const ID_UERADIO_CAPABILITY_FOR_PAGING: ProtocolIEID = ProtocolIEID(118);
    pub const ID_UERADIO_CAPABILITY_INFO_INDICATION: ProcedureCode = ProcedureCode(44);
    pub const ID_UERETENTION_INFORMATION: ProtocolIEID = ProtocolIEID(147);
    pub const ID_UESECURITY_CAPABILITIES: ProtocolIEID = ProtocolIEID(119);
    pub const ID_UETNLABINDING_RELEASE: ProcedureCode = ProcedureCode(45);
    pub const ID_UL_NGU_UP_TNLINFORMATION: ProtocolIEID = ProtocolIEID(139);
    pub const ID_UL_NGU_UP_TNLMODIFY_LIST: ProtocolIEID = ProtocolIEID(140);
    pub const ID_ULFORWARDING: ProtocolIEID = ProtocolIEID(163);
    pub const ID_ULFORWARDING_UP_TNLINFORMATION: ProtocolIEID = ProtocolIEID(164);
    pub const ID_UNAVAILABLE_GUAMILIST: ProtocolIEID = ProtocolIEID(120);
    pub const ID_UPLINK_NASTRANSPORT: ProcedureCode = ProcedureCode(46);
    pub const ID_UPLINK_NON_UEASSOCIATED_NRPPA_TRANSPORT: ProcedureCode = ProcedureCode(47);
    pub const ID_UPLINK_RANCONFIGURATION_TRANSFER: ProcedureCode = ProcedureCode(48);
    pub const ID_UPLINK_RANSTATUS_TRANSFER: ProcedureCode = ProcedureCode(49);
    pub const ID_UPLINK_UEASSOCIATED_NRPPA_TRANSPORT: ProcedureCode = ProcedureCode(50);
    pub const ID_USER_LOCATION_INFORMATION: ProtocolIEID = ProtocolIEID(121);
    pub const ID_WARNING_AREA_COORDINATES: ProtocolIEID = ProtocolIEID(141);
    pub const ID_WARNING_AREA_LIST: ProtocolIEID = ProtocolIEID(122);
    pub const ID_WARNING_MESSAGE_CONTENTS: ProtocolIEID = ProtocolIEID(123);
    pub const ID_WARNING_SECURITY_INFO: ProtocolIEID = ProtocolIEID(124);
    pub const ID_WARNING_TYPE: ProtocolIEID = ProtocolIEID(125);
    pub const ID_WRITE_REPLACE_WARNING: ProcedureCode = ProcedureCode(51);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Extension constants"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    pub static MAX_PRIVATE_IES: LazyLock<Integer> = LazyLock::new(|| Integer::from(65535i128));
    pub static MAX_PROTOCOL_EXTENSIONS: LazyLock<Integer> = LazyLock::new(|| Integer::from(65535i128));
    pub static MAX_PROTOCOL_IES: LazyLock<Integer> = LazyLock::new(|| Integer::from(65535i128));
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Lists"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    pub static MAXNOOF_ALLOWED_AREAS: LazyLock<Integer> = LazyLock::new(|| Integer::from(16i128));
    pub static MAXNOOF_ALLOWED_S_NSSAIS: LazyLock<Integer> = LazyLock::new(|| Integer::from(8i128));
    pub static MAXNOOF_AO_I: LazyLock<Integer> = LazyLock::new(|| Integer::from(64i128));
    pub static MAXNOOF_BPLMNS: LazyLock<Integer> = LazyLock::new(|| Integer::from(12i128));
    pub static MAXNOOF_CELL_IDFOR_WARNING: LazyLock<Integer> = LazyLock::new(|| Integer::from(65535i128));
    pub static MAXNOOF_CELLIN_AO_I: LazyLock<Integer> = LazyLock::new(|| Integer::from(256i128));
    pub static MAXNOOF_CELLIN_EAI: LazyLock<Integer> = LazyLock::new(|| Integer::from(65535i128));
    pub static MAXNOOF_CELLIN_TAI: LazyLock<Integer> = LazyLock::new(|| Integer::from(65535i128));
    pub static MAXNOOF_CELLS_UEMOVING_TRAJECTORY: LazyLock<Integer> = LazyLock::new(|| Integer::from(16i128));
    pub static MAXNOOF_CELLSIN_UEHISTORY_INFO: LazyLock<Integer> = LazyLock::new(|| Integer::from(16i128));
    pub static MAXNOOF_CELLSING_NB: LazyLock<Integer> = LazyLock::new(|| Integer::from(16384i128));
    pub static MAXNOOF_CELLSINNGE_NB: LazyLock<Integer> = LazyLock::new(|| Integer::from(256i128));
    pub static MAXNOOF_DRBS: LazyLock<Integer> = LazyLock::new(|| Integer::from(32i128));
    pub static MAXNOOF_E_RABS: LazyLock<Integer> = LazyLock::new(|| Integer::from(256i128));
    pub static MAXNOOF_EAIFOR_RESTART: LazyLock<Integer> = LazyLock::new(|| Integer::from(256i128));
    pub static MAXNOOF_EPLMNS: LazyLock<Integer> = LazyLock::new(|| Integer::from(15i128));
    pub static MAXNOOF_EPLMNS_PLUS_ONE: LazyLock<Integer> = LazyLock::new(|| Integer::from(16i128));
    pub static MAXNOOF_EMERGENCY_AREA_ID: LazyLock<Integer> = LazyLock::new(|| Integer::from(65535i128));
    pub static MAXNOOF_ERRORS: LazyLock<Integer> = LazyLock::new(|| Integer::from(256i128));
    pub static MAXNOOF_FORB_TACS: LazyLock<Integer> = LazyLock::new(|| Integer::from(4096i128));
    pub static MAXNOOF_MULTI_CONNECTIVITY: LazyLock<Integer> = LazyLock::new(|| Integer::from(4i128));
    pub static MAXNOOF_MULTI_CONNECTIVITY_MINUS_ONE: LazyLock<Integer> = LazyLock::new(|| Integer::from(3i128));
    pub static MAXNOOF_NGCONNECTIONS_TO_RESET: LazyLock<Integer> = LazyLock::new(|| Integer::from(65536i128));
    pub static MAXNOOF_PDUSESSIONS: LazyLock<Integer> = LazyLock::new(|| Integer::from(256i128));
    pub static MAXNOOF_PLMNS: LazyLock<Integer> = LazyLock::new(|| Integer::from(12i128));
    pub static MAXNOOF_QOS_FLOWS: LazyLock<Integer> = LazyLock::new(|| Integer::from(64i128));
    pub static MAXNOOF_RANNODEIN_AO_I: LazyLock<Integer> = LazyLock::new(|| Integer::from(64i128));
    pub static MAXNOOF_RECOMMENDED_CELLS: LazyLock<Integer> = LazyLock::new(|| Integer::from(16i128));
    pub static MAXNOOF_RECOMMENDED_RANNODES: LazyLock<Integer> = LazyLock::new(|| Integer::from(16i128));
    pub static MAXNOOF_SERVED_GUAMIS: LazyLock<Integer> = LazyLock::new(|| Integer::from(256i128));
    pub static MAXNOOF_SLICE_ITEMS: LazyLock<Integer> = LazyLock::new(|| Integer::from(1024i128));
    pub static MAXNOOF_TACS: LazyLock<Integer> = LazyLock::new(|| Integer::from(256i128));
    pub static MAXNOOF_TAIFOR_INACTIVE: LazyLock<Integer> = LazyLock::new(|| Integer::from(16i128));
    pub static MAXNOOF_TAIFOR_PAGING: LazyLock<Integer> = LazyLock::new(|| Integer::from(16i128));
    pub static MAXNOOF_TAIFOR_RESTART: LazyLock<Integer> = LazyLock::new(|| Integer::from(2048i128));
    pub static MAXNOOF_TAIFOR_WARNING: LazyLock<Integer> = LazyLock::new(|| Integer::from(65535i128));
    pub static MAXNOOF_TAIIN_AO_I: LazyLock<Integer> = LazyLock::new(|| Integer::from(16i128));
    pub static MAXNOOF_TNLASSOCIATIONS: LazyLock<Integer> = LazyLock::new(|| Integer::from(32i128));
    pub static MAXNOOF_TIME_PERIODS: LazyLock<Integer> = LazyLock::new(|| Integer::from(2i128));
    pub static MAXNOOF_XN_EXT_TLAS: LazyLock<Integer> = LazyLock::new(|| Integer::from(16i128));
    pub static MAXNOOF_XN_GTP_TLAS: LazyLock<Integer> = LazyLock::new(|| Integer::from(16i128));
    pub static MAXNOOF_XN_TLAS: LazyLock<Integer> = LazyLock::new(|| Integer::from(2i128));
}
#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, unused, clippy::too_many_arguments)]
pub mod ngap_containers {
    extern crate alloc;
    use super::ngap_common_data_types::*;
    use super::ngap_constants::*;
    use core::borrow::Borrow;
    use rasn::prelude::*;
    use std::sync::LazyLock;
}
#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, unused, clippy::too_many_arguments)]
pub mod ngap_ies {
    extern crate alloc;
    use super::ngap_common_data_types::*;
    use super::ngap_common_data_types::*;
    use super::ngap_constants::*;
    use super::ngap_containers::*;
    use core::borrow::Borrow;
    use rasn::prelude::*;
    use std::sync::LazyLock;
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAMFTNLAssociationSetupItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAMFTNLAssociationSetupItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAMFTNLAssociationSetupItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAMFTNLAssociationSetupItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAMFTNLAssociationSetupItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AMFTNLAssociationSetupItemIEExtensions(pub SequenceOf<AnonymousAMFTNLAssociationSetupItemIEExtensions>);
    aper_sequence_of!(
        AMFTNLAssociationSetupItemIEExtensions,
        AnonymousAMFTNLAssociationSetupItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "AMF-TNLAssociationSetupItem")]
    #[non_exhaustive]
    pub struct AMFTNLAssociationSetupItem {
        #[rasn(identifier = "aMF-TNLAssociationAddress")]
        pub a_mf_tnlassociation_address: CPTransportLayerInformation,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AMFTNLAssociationSetupItemIEExtensions>,
    }
    impl AMFTNLAssociationSetupItem {
        pub fn new(
            a_mf_tnlassociation_address: CPTransportLayerInformation,
            i_e_extensions: Option<AMFTNLAssociationSetupItemIEExtensions>,
        ) -> Self {
            Self { a_mf_tnlassociation_address, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=32"), identifier = "AMF-TNLAssociationSetupList")]
    pub struct AMFTNLAssociationSetupList(pub SequenceOf<AMFTNLAssociationSetupItem>);
    aper_sequence_of!(AMFTNLAssociationSetupList, AMFTNLAssociationSetupItem, 1, 32);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAMFTNLAssociationToAddItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAMFTNLAssociationToAddItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAMFTNLAssociationToAddItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAMFTNLAssociationToAddItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAMFTNLAssociationToAddItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AMFTNLAssociationToAddItemIEExtensions(pub SequenceOf<AnonymousAMFTNLAssociationToAddItemIEExtensions>);
    aper_sequence_of!(
        AMFTNLAssociationToAddItemIEExtensions,
        AnonymousAMFTNLAssociationToAddItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "AMF-TNLAssociationToAddItem")]
    #[non_exhaustive]
    pub struct AMFTNLAssociationToAddItem {
        #[rasn(identifier = "aMF-TNLAssociationAddress")]
        pub a_mf_tnlassociation_address: CPTransportLayerInformation,
        #[rasn(identifier = "tNLAssociationUsage")]
        pub t_nlassociation_usage: Option<TNLAssociationUsage>,
        #[rasn(identifier = "tNLAddressWeightFactor")]
        pub t_nladdress_weight_factor: TNLAddressWeightFactor,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AMFTNLAssociationToAddItemIEExtensions>,
    }
    impl AMFTNLAssociationToAddItem {
        pub fn new(
            a_mf_tnlassociation_address: CPTransportLayerInformation,
            t_nlassociation_usage: Option<TNLAssociationUsage>,
            t_nladdress_weight_factor: TNLAddressWeightFactor,
            i_e_extensions: Option<AMFTNLAssociationToAddItemIEExtensions>,
        ) -> Self {
            Self { a_mf_tnlassociation_address, t_nlassociation_usage, t_nladdress_weight_factor, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=32"), identifier = "AMF-TNLAssociationToAddList")]
    pub struct AMFTNLAssociationToAddList(pub SequenceOf<AMFTNLAssociationToAddItem>);
    aper_sequence_of!(AMFTNLAssociationToAddList, AMFTNLAssociationToAddItem, 1, 32);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAMFTNLAssociationToRemoveItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAMFTNLAssociationToRemoveItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAMFTNLAssociationToRemoveItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAMFTNLAssociationToRemoveItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAMFTNLAssociationToRemoveItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AMFTNLAssociationToRemoveItemIEExtensions(
        pub SequenceOf<AnonymousAMFTNLAssociationToRemoveItemIEExtensions>,
    );
    aper_sequence_of!(
        AMFTNLAssociationToRemoveItemIEExtensions,
        AnonymousAMFTNLAssociationToRemoveItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "AMF-TNLAssociationToRemoveItem")]
    #[non_exhaustive]
    pub struct AMFTNLAssociationToRemoveItem {
        #[rasn(identifier = "aMF-TNLAssociationAddress")]
        pub a_mf_tnlassociation_address: CPTransportLayerInformation,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AMFTNLAssociationToRemoveItemIEExtensions>,
    }
    impl AMFTNLAssociationToRemoveItem {
        pub fn new(
            a_mf_tnlassociation_address: CPTransportLayerInformation,
            i_e_extensions: Option<AMFTNLAssociationToRemoveItemIEExtensions>,
        ) -> Self {
            Self { a_mf_tnlassociation_address, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=32"), identifier = "AMF-TNLAssociationToRemoveList")]
    pub struct AMFTNLAssociationToRemoveList(pub SequenceOf<AMFTNLAssociationToRemoveItem>);
    aper_sequence_of!(AMFTNLAssociationToRemoveList, AMFTNLAssociationToRemoveItem, 1, 32);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAMFTNLAssociationToUpdateItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAMFTNLAssociationToUpdateItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAMFTNLAssociationToUpdateItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAMFTNLAssociationToUpdateItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAMFTNLAssociationToUpdateItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AMFTNLAssociationToUpdateItemIEExtensions(
        pub SequenceOf<AnonymousAMFTNLAssociationToUpdateItemIEExtensions>,
    );
    aper_sequence_of!(
        AMFTNLAssociationToUpdateItemIEExtensions,
        AnonymousAMFTNLAssociationToUpdateItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "AMF-TNLAssociationToUpdateItem")]
    #[non_exhaustive]
    pub struct AMFTNLAssociationToUpdateItem {
        #[rasn(identifier = "aMF-TNLAssociationAddress")]
        pub a_mf_tnlassociation_address: CPTransportLayerInformation,
        #[rasn(identifier = "tNLAssociationUsage")]
        pub t_nlassociation_usage: Option<TNLAssociationUsage>,
        #[rasn(identifier = "tNLAddressWeightFactor")]
        pub t_nladdress_weight_factor: Option<TNLAddressWeightFactor>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AMFTNLAssociationToUpdateItemIEExtensions>,
    }
    impl AMFTNLAssociationToUpdateItem {
        pub fn new(
            a_mf_tnlassociation_address: CPTransportLayerInformation,
            t_nlassociation_usage: Option<TNLAssociationUsage>,
            t_nladdress_weight_factor: Option<TNLAddressWeightFactor>,
            i_e_extensions: Option<AMFTNLAssociationToUpdateItemIEExtensions>,
        ) -> Self {
            Self { a_mf_tnlassociation_address, t_nlassociation_usage, t_nladdress_weight_factor, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=32"), identifier = "AMF-TNLAssociationToUpdateList")]
    pub struct AMFTNLAssociationToUpdateList(pub SequenceOf<AMFTNLAssociationToUpdateItem>);
    aper_sequence_of!(AMFTNLAssociationToUpdateList, AMFTNLAssociationToUpdateItem, 1, 32);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "AMF-UE-NGAP-ID", value("0..=1099511627775"))]
    pub struct AMFUENGAPID(pub u64);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=150", extensible))]
    pub struct AMFName(pub PrintableString);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AMFPagingTargetChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct AMFPagingTargetChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AMFPagingTargetChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl AMFPagingTargetChoiceExtensions {
        pub fn new(id: u16, criticality: AMFPagingTargetChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum AMFPagingTarget {
        globalRANNodeID(GlobalRANNodeID),
        tAI(TAI),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(AMFPagingTargetChoiceExtensions),
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct AMFPointer(pub FixedBitString<6usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct AMFRegionID(pub FixedBitString<8usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct AMFSetID(pub FixedBitString<10usize>);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAdditionalDLUPTNLInformationForHOItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAdditionalDLUPTNLInformationForHOItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAdditionalDLUPTNLInformationForHOItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAdditionalDLUPTNLInformationForHOItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAdditionalDLUPTNLInformationForHOItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AdditionalDLUPTNLInformationForHOItemIEExtensions(
        pub SequenceOf<AnonymousAdditionalDLUPTNLInformationForHOItemIEExtensions>,
    );
    aper_sequence_of!(
        AdditionalDLUPTNLInformationForHOItemIEExtensions,
        AnonymousAdditionalDLUPTNLInformationForHOItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AdditionalDLUPTNLInformationForHOItem {
        #[rasn(identifier = "additionalDL-NGU-UP-TNLInformation")]
        pub additional_dl_ngu_up_tnlinformation: UPTransportLayerInformation,
        #[rasn(identifier = "additionalQosFlowSetupResponseList")]
        pub additional_qos_flow_setup_response_list: QosFlowListWithDataForwarding,
        #[rasn(identifier = "additionalDLForwardingUPTNLInformation")]
        pub additional_dlforwarding_uptnlinformation: Option<UPTransportLayerInformation>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AdditionalDLUPTNLInformationForHOItemIEExtensions>,
    }
    impl AdditionalDLUPTNLInformationForHOItem {
        pub fn new(
            additional_dl_ngu_up_tnlinformation: UPTransportLayerInformation,
            additional_qos_flow_setup_response_list: QosFlowListWithDataForwarding,
            additional_dlforwarding_uptnlinformation: Option<UPTransportLayerInformation>,
            i_e_extensions: Option<AdditionalDLUPTNLInformationForHOItemIEExtensions>,
        ) -> Self {
            Self {
                additional_dl_ngu_up_tnlinformation,
                additional_qos_flow_setup_response_list,
                additional_dlforwarding_uptnlinformation,
                i_e_extensions,
            }
        }
    }
    #[doc = " A"]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=3"))]
    pub struct AdditionalDLUPTNLInformationForHOList(pub SequenceOf<AdditionalDLUPTNLInformationForHOItem>);
    aper_sequence_of!(AdditionalDLUPTNLInformationForHOList, AdditionalDLUPTNLInformationForHOItem, 1, 3);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum AdditionalQosFlowInformation {
        #[rasn(identifier = "more-likely")]
        more_likely = 0,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAllocationAndRetentionPriorityIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAllocationAndRetentionPriorityIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAllocationAndRetentionPriorityIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAllocationAndRetentionPriorityIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAllocationAndRetentionPriorityIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AllocationAndRetentionPriorityIEExtensions(
        pub SequenceOf<AnonymousAllocationAndRetentionPriorityIEExtensions>,
    );
    aper_sequence_of!(
        AllocationAndRetentionPriorityIEExtensions,
        AnonymousAllocationAndRetentionPriorityIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AllocationAndRetentionPriority {
        #[rasn(identifier = "priorityLevelARP")]
        pub priority_level_arp: PriorityLevelARP,
        #[rasn(identifier = "pre-emptionCapability")]
        pub pre_emption_capability: PreEmptionCapability,
        #[rasn(identifier = "pre-emptionVulnerability")]
        pub pre_emption_vulnerability: PreEmptionVulnerability,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AllocationAndRetentionPriorityIEExtensions>,
    }
    impl AllocationAndRetentionPriority {
        pub fn new(
            priority_level_arp: PriorityLevelARP,
            pre_emption_capability: PreEmptionCapability,
            pre_emption_vulnerability: PreEmptionVulnerability,
            i_e_extensions: Option<AllocationAndRetentionPriorityIEExtensions>,
        ) -> Self {
            Self { priority_level_arp, pre_emption_capability, pre_emption_vulnerability, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=8"))]
    pub struct AllowedNSSAI(pub SequenceOf<AllowedNSSAIItem>);
    aper_sequence_of!(AllowedNSSAI, AllowedNSSAIItem, 1, 8);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAllowedNSSAIItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAllowedNSSAIItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAllowedNSSAIItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAllowedNSSAIItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAllowedNSSAIItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AllowedNSSAIItemIEExtensions(pub SequenceOf<AnonymousAllowedNSSAIItemIEExtensions>);
    aper_sequence_of!(AllowedNSSAIItemIEExtensions, AnonymousAllowedNSSAIItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "AllowedNSSAI-Item")]
    #[non_exhaustive]
    pub struct AllowedNSSAIItem {
        #[rasn(identifier = "s-NSSAI")]
        pub s_nssai: SNSSAI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AllowedNSSAIItemIEExtensions>,
    }
    impl AllowedNSSAIItem {
        pub fn new(s_nssai: SNSSAI, i_e_extensions: Option<AllowedNSSAIItemIEExtensions>) -> Self {
            Self { s_nssai, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"))]
    pub struct AllowedTACs(pub SequenceOf<TAC>);
    aper_sequence_of!(AllowedTACs, TAC, 1, 16);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAreaOfInterestIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAreaOfInterestIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAreaOfInterestIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAreaOfInterestIEExtensions {
        pub fn new(id: u16, criticality: AnonymousAreaOfInterestIEExtensionsCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AreaOfInterestIEExtensions(pub SequenceOf<AnonymousAreaOfInterestIEExtensions>);
    aper_sequence_of!(AreaOfInterestIEExtensions, AnonymousAreaOfInterestIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AreaOfInterest {
        #[rasn(identifier = "areaOfInterestTAIList")]
        pub area_of_interest_tailist: Option<AreaOfInterestTAIList>,
        #[rasn(identifier = "areaOfInterestCellList")]
        pub area_of_interest_cell_list: Option<AreaOfInterestCellList>,
        #[rasn(identifier = "areaOfInterestRANNodeList")]
        pub area_of_interest_rannode_list: Option<AreaOfInterestRANNodeList>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AreaOfInterestIEExtensions>,
    }
    impl AreaOfInterest {
        pub fn new(
            area_of_interest_tailist: Option<AreaOfInterestTAIList>,
            area_of_interest_cell_list: Option<AreaOfInterestCellList>,
            area_of_interest_rannode_list: Option<AreaOfInterestRANNodeList>,
            i_e_extensions: Option<AreaOfInterestIEExtensions>,
        ) -> Self {
            Self { area_of_interest_tailist, area_of_interest_cell_list, area_of_interest_rannode_list, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAreaOfInterestCellItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAreaOfInterestCellItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAreaOfInterestCellItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAreaOfInterestCellItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAreaOfInterestCellItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AreaOfInterestCellItemIEExtensions(pub SequenceOf<AnonymousAreaOfInterestCellItemIEExtensions>);
    aper_sequence_of!(AreaOfInterestCellItemIEExtensions, AnonymousAreaOfInterestCellItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AreaOfInterestCellItem {
        #[rasn(identifier = "nGRAN-CGI")]
        pub n_gran_cgi: NGRANCGI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AreaOfInterestCellItemIEExtensions>,
    }
    impl AreaOfInterestCellItem {
        pub fn new(n_gran_cgi: NGRANCGI, i_e_extensions: Option<AreaOfInterestCellItemIEExtensions>) -> Self {
            Self { n_gran_cgi, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct AreaOfInterestCellList(pub SequenceOf<AreaOfInterestCellItem>);
    aper_sequence_of!(AreaOfInterestCellList, AreaOfInterestCellItem, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAreaOfInterestItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAreaOfInterestItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAreaOfInterestItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAreaOfInterestItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAreaOfInterestItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AreaOfInterestItemIEExtensions(pub SequenceOf<AnonymousAreaOfInterestItemIEExtensions>);
    aper_sequence_of!(AreaOfInterestItemIEExtensions, AnonymousAreaOfInterestItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AreaOfInterestItem {
        #[rasn(identifier = "areaOfInterest")]
        pub area_of_interest: AreaOfInterest,
        #[rasn(identifier = "locationReportingReferenceID")]
        pub location_reporting_reference_id: LocationReportingReferenceID,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AreaOfInterestItemIEExtensions>,
    }
    impl AreaOfInterestItem {
        pub fn new(
            area_of_interest: AreaOfInterest,
            location_reporting_reference_id: LocationReportingReferenceID,
            i_e_extensions: Option<AreaOfInterestItemIEExtensions>,
        ) -> Self {
            Self { area_of_interest, location_reporting_reference_id, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct AreaOfInterestList(pub SequenceOf<AreaOfInterestItem>);
    aper_sequence_of!(AreaOfInterestList, AreaOfInterestItem, 1, 64);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAreaOfInterestRANNodeItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAreaOfInterestRANNodeItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAreaOfInterestRANNodeItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAreaOfInterestRANNodeItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAreaOfInterestRANNodeItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AreaOfInterestRANNodeItemIEExtensions(pub SequenceOf<AnonymousAreaOfInterestRANNodeItemIEExtensions>);
    aper_sequence_of!(AreaOfInterestRANNodeItemIEExtensions, AnonymousAreaOfInterestRANNodeItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AreaOfInterestRANNodeItem {
        #[rasn(identifier = "globalRANNodeID")]
        pub global_rannode_id: GlobalRANNodeID,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AreaOfInterestRANNodeItemIEExtensions>,
    }
    impl AreaOfInterestRANNodeItem {
        pub fn new(
            global_rannode_id: GlobalRANNodeID,
            i_e_extensions: Option<AreaOfInterestRANNodeItemIEExtensions>,
        ) -> Self {
            Self { global_rannode_id, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct AreaOfInterestRANNodeList(pub SequenceOf<AreaOfInterestRANNodeItem>);
    aper_sequence_of!(AreaOfInterestRANNodeList, AreaOfInterestRANNodeItem, 1, 64);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAreaOfInterestTAIItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAreaOfInterestTAIItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAreaOfInterestTAIItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAreaOfInterestTAIItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAreaOfInterestTAIItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AreaOfInterestTAIItemIEExtensions(pub SequenceOf<AnonymousAreaOfInterestTAIItemIEExtensions>);
    aper_sequence_of!(AreaOfInterestTAIItemIEExtensions, AnonymousAreaOfInterestTAIItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AreaOfInterestTAIItem {
        #[rasn(identifier = "tAI")]
        pub t_ai: TAI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AreaOfInterestTAIItemIEExtensions>,
    }
    impl AreaOfInterestTAIItem {
        pub fn new(t_ai: TAI, i_e_extensions: Option<AreaOfInterestTAIItemIEExtensions>) -> Self {
            Self { t_ai, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"))]
    pub struct AreaOfInterestTAIList(pub SequenceOf<AreaOfInterestTAIItem>);
    aper_sequence_of!(AreaOfInterestTAIList, AreaOfInterestTAIItem, 1, 16);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAssistanceDataForPagingIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAssistanceDataForPagingIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAssistanceDataForPagingIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAssistanceDataForPagingIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAssistanceDataForPagingIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AssistanceDataForPagingIEExtensions(pub SequenceOf<AnonymousAssistanceDataForPagingIEExtensions>);
    aper_sequence_of!(AssistanceDataForPagingIEExtensions, AnonymousAssistanceDataForPagingIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AssistanceDataForPaging {
        #[rasn(identifier = "assistanceDataForRecommendedCells")]
        pub assistance_data_for_recommended_cells: Option<AssistanceDataForRecommendedCells>,
        #[rasn(identifier = "pagingAttemptInformation")]
        pub paging_attempt_information: Option<PagingAttemptInformation>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AssistanceDataForPagingIEExtensions>,
    }
    impl AssistanceDataForPaging {
        pub fn new(
            assistance_data_for_recommended_cells: Option<AssistanceDataForRecommendedCells>,
            paging_attempt_information: Option<PagingAttemptInformation>,
            i_e_extensions: Option<AssistanceDataForPagingIEExtensions>,
        ) -> Self {
            Self { assistance_data_for_recommended_cells, paging_attempt_information, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAssistanceDataForRecommendedCellsIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAssistanceDataForRecommendedCellsIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAssistanceDataForRecommendedCellsIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAssistanceDataForRecommendedCellsIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAssistanceDataForRecommendedCellsIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AssistanceDataForRecommendedCellsIEExtensions(
        pub SequenceOf<AnonymousAssistanceDataForRecommendedCellsIEExtensions>,
    );
    aper_sequence_of!(
        AssistanceDataForRecommendedCellsIEExtensions,
        AnonymousAssistanceDataForRecommendedCellsIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AssistanceDataForRecommendedCells {
        #[rasn(identifier = "recommendedCellsForPaging")]
        pub recommended_cells_for_paging: RecommendedCellsForPaging,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AssistanceDataForRecommendedCellsIEExtensions>,
    }
    impl AssistanceDataForRecommendedCells {
        pub fn new(
            recommended_cells_for_paging: RecommendedCellsForPaging,
            i_e_extensions: Option<AssistanceDataForRecommendedCellsIEExtensions>,
        ) -> Self {
            Self { recommended_cells_for_paging, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum AssociatedQosFlowItemQosFlowMappingIndication {
        ul = 0,
        dl = 1,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAssociatedQosFlowItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAssociatedQosFlowItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAssociatedQosFlowItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousAssociatedQosFlowItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousAssociatedQosFlowItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct AssociatedQosFlowItemIEExtensions(pub SequenceOf<AnonymousAssociatedQosFlowItemIEExtensions>);
    aper_sequence_of!(AssociatedQosFlowItemIEExtensions, AnonymousAssociatedQosFlowItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AssociatedQosFlowItem {
        #[rasn(identifier = "qosFlowIdentifier")]
        pub qos_flow_identifier: QosFlowIdentifier,
        #[rasn(identifier = "qosFlowMappingIndication")]
        pub qos_flow_mapping_indication: Option<AssociatedQosFlowItemQosFlowMappingIndication>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<AssociatedQosFlowItemIEExtensions>,
    }
    impl AssociatedQosFlowItem {
        pub fn new(
            qos_flow_identifier: QosFlowIdentifier,
            qos_flow_mapping_indication: Option<AssociatedQosFlowItemQosFlowMappingIndication>,
            i_e_extensions: Option<AssociatedQosFlowItemIEExtensions>,
        ) -> Self {
            Self { qos_flow_identifier, qos_flow_mapping_indication, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct AssociatedQosFlowList(pub SequenceOf<AssociatedQosFlowItem>);
    aper_sequence_of!(AssociatedQosFlowList, AssociatedQosFlowItem, 1, 64);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=4095", extensible))]
    pub struct AveragingWindow(pub Integer);
    #[doc = " B"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=4000000000000", extensible))]
    pub struct BitRate(pub Integer);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum BroadcastCancelledAreaListChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct BroadcastCancelledAreaListChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: BroadcastCancelledAreaListChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl BroadcastCancelledAreaListChoiceExtensions {
        pub fn new(id: u16, criticality: BroadcastCancelledAreaListChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum BroadcastCancelledAreaList {
        cellIDCancelledEUTRA(CellIDCancelledEUTRA),
        tAICancelledEUTRA(TAICancelledEUTRA),
        emergencyAreaIDCancelledEUTRA(EmergencyAreaIDCancelledEUTRA),
        cellIDCancelledNR(CellIDCancelledNR),
        tAICancelledNR(TAICancelledNR),
        emergencyAreaIDCancelledNR(EmergencyAreaIDCancelledNR),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(BroadcastCancelledAreaListChoiceExtensions),
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum BroadcastCompletedAreaListChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct BroadcastCompletedAreaListChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: BroadcastCompletedAreaListChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl BroadcastCompletedAreaListChoiceExtensions {
        pub fn new(id: u16, criticality: BroadcastCompletedAreaListChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum BroadcastCompletedAreaList {
        cellIDBroadcastEUTRA(CellIDBroadcastEUTRA),
        tAIBroadcastEUTRA(TAIBroadcastEUTRA),
        emergencyAreaIDBroadcastEUTRA(EmergencyAreaIDBroadcastEUTRA),
        cellIDBroadcastNR(CellIDBroadcastNR),
        tAIBroadcastNR(TAIBroadcastNR),
        emergencyAreaIDBroadcastNR(EmergencyAreaIDBroadcastNR),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(BroadcastCompletedAreaListChoiceExtensions),
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousBroadcastPLMNItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousBroadcastPLMNItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousBroadcastPLMNItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousBroadcastPLMNItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousBroadcastPLMNItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct BroadcastPLMNItemIEExtensions(pub SequenceOf<AnonymousBroadcastPLMNItemIEExtensions>);
    aper_sequence_of!(BroadcastPLMNItemIEExtensions, AnonymousBroadcastPLMNItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct BroadcastPLMNItem {
        #[rasn(identifier = "pLMNIdentity")]
        pub p_lmnidentity: PLMNIdentity,
        #[rasn(identifier = "tAISliceSupportList")]
        pub t_aislice_support_list: SliceSupportList,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<BroadcastPLMNItemIEExtensions>,
    }
    impl BroadcastPLMNItem {
        pub fn new(
            p_lmnidentity: PLMNIdentity,
            t_aislice_support_list: SliceSupportList,
            i_e_extensions: Option<BroadcastPLMNItemIEExtensions>,
        ) -> Self {
            Self { p_lmnidentity, t_aislice_support_list, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=12"))]
    pub struct BroadcastPLMNList(pub SequenceOf<BroadcastPLMNItem>);
    aper_sequence_of!(BroadcastPLMNList, BroadcastPLMNItem, 1, 12);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCNAssistedRANTuningIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCNAssistedRANTuningIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCNAssistedRANTuningIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCNAssistedRANTuningIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCNAssistedRANTuningIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CNAssistedRANTuningIEExtensions(pub SequenceOf<AnonymousCNAssistedRANTuningIEExtensions>);
    aper_sequence_of!(CNAssistedRANTuningIEExtensions, AnonymousCNAssistedRANTuningIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct CNAssistedRANTuning {
        #[rasn(identifier = "expectedUEBehaviour")]
        pub expected_uebehaviour: Option<ExpectedUEBehaviour>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CNAssistedRANTuningIEExtensions>,
    }
    impl CNAssistedRANTuning {
        pub fn new(
            expected_uebehaviour: Option<ExpectedUEBehaviour>,
            i_e_extensions: Option<CNAssistedRANTuningIEExtensions>,
        ) -> Self {
            Self { expected_uebehaviour, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=15"))]
    pub struct CNTypeRestrictionsForEquivalent(pub SequenceOf<CNTypeRestrictionsForEquivalentItem>);
    aper_sequence_of!(CNTypeRestrictionsForEquivalent, CNTypeRestrictionsForEquivalentItem, 1, 15);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum CNTypeRestrictionsForEquivalentItemCnType {
        #[rasn(identifier = "epc-forbidden")]
        epc_forbidden = 0,
        #[rasn(identifier = "fiveGC-forbidden")]
        fiveGC_forbidden = 1,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCNTypeRestrictionsForEquivalentItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCNTypeRestrictionsForEquivalentItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCNTypeRestrictionsForEquivalentItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCNTypeRestrictionsForEquivalentItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCNTypeRestrictionsForEquivalentItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CNTypeRestrictionsForEquivalentItemIEExtensions(
        pub SequenceOf<AnonymousCNTypeRestrictionsForEquivalentItemIEExtensions>,
    );
    aper_sequence_of!(
        CNTypeRestrictionsForEquivalentItemIEExtensions,
        AnonymousCNTypeRestrictionsForEquivalentItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct CNTypeRestrictionsForEquivalentItem {
        #[rasn(identifier = "plmnIdentity")]
        pub plmn_identity: PLMNIdentity,
        #[rasn(identifier = "cn-Type")]
        pub cn_type: CNTypeRestrictionsForEquivalentItemCnType,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CNTypeRestrictionsForEquivalentItemIEExtensions>,
    }
    impl CNTypeRestrictionsForEquivalentItem {
        pub fn new(
            plmn_identity: PLMNIdentity,
            cn_type: CNTypeRestrictionsForEquivalentItemCnType,
            i_e_extensions: Option<CNTypeRestrictionsForEquivalentItemIEExtensions>,
        ) -> Self {
            Self { plmn_identity, cn_type, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum CNTypeRestrictionsForServing {
        #[rasn(identifier = "epc-forbidden")]
        epc_forbidden = 0,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCOUNTValueForPDCPSN12IEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCOUNTValueForPDCPSN12IEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCOUNTValueForPDCPSN12IEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCOUNTValueForPDCPSN12IEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCOUNTValueForPDCPSN12IEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct COUNTValueForPDCPSN12IEExtensions(pub SequenceOf<AnonymousCOUNTValueForPDCPSN12IEExtensions>);
    aper_sequence_of!(COUNTValueForPDCPSN12IEExtensions, AnonymousCOUNTValueForPDCPSN12IEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "COUNTValueForPDCP-SN12")]
    #[non_exhaustive]
    pub struct COUNTValueForPDCPSN12 {
        #[rasn(value("0..=4095"), identifier = "pDCP-SN12")]
        pub p_dcp_sn12: u16,
        #[rasn(value("0..=1048575"), identifier = "hFN-PDCP-SN12")]
        pub h_fn_pdcp_sn12: u32,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<COUNTValueForPDCPSN12IEExtensions>,
    }
    impl COUNTValueForPDCPSN12 {
        pub fn new(
            p_dcp_sn12: u16,
            h_fn_pdcp_sn12: u32,
            i_e_extensions: Option<COUNTValueForPDCPSN12IEExtensions>,
        ) -> Self {
            Self { p_dcp_sn12, h_fn_pdcp_sn12, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCOUNTValueForPDCPSN18IEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCOUNTValueForPDCPSN18IEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCOUNTValueForPDCPSN18IEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCOUNTValueForPDCPSN18IEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCOUNTValueForPDCPSN18IEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct COUNTValueForPDCPSN18IEExtensions(pub SequenceOf<AnonymousCOUNTValueForPDCPSN18IEExtensions>);
    aper_sequence_of!(COUNTValueForPDCPSN18IEExtensions, AnonymousCOUNTValueForPDCPSN18IEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "COUNTValueForPDCP-SN18")]
    #[non_exhaustive]
    pub struct COUNTValueForPDCPSN18 {
        #[rasn(value("0..=262143"), identifier = "pDCP-SN18")]
        pub p_dcp_sn18: u32,
        #[rasn(value("0..=16383"), identifier = "hFN-PDCP-SN18")]
        pub h_fn_pdcp_sn18: u16,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<COUNTValueForPDCPSN18IEExtensions>,
    }
    impl COUNTValueForPDCPSN18 {
        pub fn new(
            p_dcp_sn18: u32,
            h_fn_pdcp_sn18: u16,
            i_e_extensions: Option<COUNTValueForPDCPSN18IEExtensions>,
        ) -> Self {
            Self { p_dcp_sn18, h_fn_pdcp_sn18, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum CPTransportLayerInformationChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct CPTransportLayerInformationChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: CPTransportLayerInformationChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl CPTransportLayerInformationChoiceExtensions {
        pub fn new(id: u16, criticality: CPTransportLayerInformationChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum CPTransportLayerInformation {
        endpointIPAddress(TransportLayerAddress),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(CPTransportLayerInformationChoiceExtensions),
    }
    #[doc = " C"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum CancelAllWarningMessages {
        #[rasn(identifier = "true")]
        R_true = 0,
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"), identifier = "CancelledCellsInEAI-EUTRA")]
    pub struct CancelledCellsInEAIEUTRA(pub SequenceOf<CancelledCellsInEAIEUTRAItem>);
    aper_sequence_of!(CancelledCellsInEAIEUTRA, CancelledCellsInEAIEUTRAItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCancelledCellsInEAIEUTRAItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCancelledCellsInEAIEUTRAItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCancelledCellsInEAIEUTRAItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCancelledCellsInEAIEUTRAItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCancelledCellsInEAIEUTRAItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CancelledCellsInEAIEUTRAItemIEExtensions(
        pub SequenceOf<AnonymousCancelledCellsInEAIEUTRAItemIEExtensions>,
    );
    aper_sequence_of!(
        CancelledCellsInEAIEUTRAItemIEExtensions,
        AnonymousCancelledCellsInEAIEUTRAItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "CancelledCellsInEAI-EUTRA-Item")]
    #[non_exhaustive]
    pub struct CancelledCellsInEAIEUTRAItem {
        #[rasn(identifier = "eUTRA-CGI")]
        pub e_utra_cgi: EUTRACGI,
        #[rasn(identifier = "numberOfBroadcasts")]
        pub number_of_broadcasts: NumberOfBroadcasts,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CancelledCellsInEAIEUTRAItemIEExtensions>,
    }
    impl CancelledCellsInEAIEUTRAItem {
        pub fn new(
            e_utra_cgi: EUTRACGI,
            number_of_broadcasts: NumberOfBroadcasts,
            i_e_extensions: Option<CancelledCellsInEAIEUTRAItemIEExtensions>,
        ) -> Self {
            Self { e_utra_cgi, number_of_broadcasts, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"), identifier = "CancelledCellsInEAI-NR")]
    pub struct CancelledCellsInEAINR(pub SequenceOf<CancelledCellsInEAINRItem>);
    aper_sequence_of!(CancelledCellsInEAINR, CancelledCellsInEAINRItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCancelledCellsInEAINRItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCancelledCellsInEAINRItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCancelledCellsInEAINRItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCancelledCellsInEAINRItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCancelledCellsInEAINRItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CancelledCellsInEAINRItemIEExtensions(pub SequenceOf<AnonymousCancelledCellsInEAINRItemIEExtensions>);
    aper_sequence_of!(CancelledCellsInEAINRItemIEExtensions, AnonymousCancelledCellsInEAINRItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "CancelledCellsInEAI-NR-Item")]
    #[non_exhaustive]
    pub struct CancelledCellsInEAINRItem {
        #[rasn(identifier = "nR-CGI")]
        pub n_r_cgi: NRCGI,
        #[rasn(identifier = "numberOfBroadcasts")]
        pub number_of_broadcasts: NumberOfBroadcasts,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CancelledCellsInEAINRItemIEExtensions>,
    }
    impl CancelledCellsInEAINRItem {
        pub fn new(
            n_r_cgi: NRCGI,
            number_of_broadcasts: NumberOfBroadcasts,
            i_e_extensions: Option<CancelledCellsInEAINRItemIEExtensions>,
        ) -> Self {
            Self { n_r_cgi, number_of_broadcasts, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"), identifier = "CancelledCellsInTAI-EUTRA")]
    pub struct CancelledCellsInTAIEUTRA(pub SequenceOf<CancelledCellsInTAIEUTRAItem>);
    aper_sequence_of!(CancelledCellsInTAIEUTRA, CancelledCellsInTAIEUTRAItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCancelledCellsInTAIEUTRAItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCancelledCellsInTAIEUTRAItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCancelledCellsInTAIEUTRAItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCancelledCellsInTAIEUTRAItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCancelledCellsInTAIEUTRAItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CancelledCellsInTAIEUTRAItemIEExtensions(
        pub SequenceOf<AnonymousCancelledCellsInTAIEUTRAItemIEExtensions>,
    );
    aper_sequence_of!(
        CancelledCellsInTAIEUTRAItemIEExtensions,
        AnonymousCancelledCellsInTAIEUTRAItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "CancelledCellsInTAI-EUTRA-Item")]
    #[non_exhaustive]
    pub struct CancelledCellsInTAIEUTRAItem {
        #[rasn(identifier = "eUTRA-CGI")]
        pub e_utra_cgi: EUTRACGI,
        #[rasn(identifier = "numberOfBroadcasts")]
        pub number_of_broadcasts: NumberOfBroadcasts,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CancelledCellsInTAIEUTRAItemIEExtensions>,
    }
    impl CancelledCellsInTAIEUTRAItem {
        pub fn new(
            e_utra_cgi: EUTRACGI,
            number_of_broadcasts: NumberOfBroadcasts,
            i_e_extensions: Option<CancelledCellsInTAIEUTRAItemIEExtensions>,
        ) -> Self {
            Self { e_utra_cgi, number_of_broadcasts, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"), identifier = "CancelledCellsInTAI-NR")]
    pub struct CancelledCellsInTAINR(pub SequenceOf<CancelledCellsInTAINRItem>);
    aper_sequence_of!(CancelledCellsInTAINR, CancelledCellsInTAINRItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCancelledCellsInTAINRItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCancelledCellsInTAINRItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCancelledCellsInTAINRItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCancelledCellsInTAINRItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCancelledCellsInTAINRItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CancelledCellsInTAINRItemIEExtensions(pub SequenceOf<AnonymousCancelledCellsInTAINRItemIEExtensions>);
    aper_sequence_of!(CancelledCellsInTAINRItemIEExtensions, AnonymousCancelledCellsInTAINRItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "CancelledCellsInTAI-NR-Item")]
    #[non_exhaustive]
    pub struct CancelledCellsInTAINRItem {
        #[rasn(identifier = "nR-CGI")]
        pub n_r_cgi: NRCGI,
        #[rasn(identifier = "numberOfBroadcasts")]
        pub number_of_broadcasts: NumberOfBroadcasts,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CancelledCellsInTAINRItemIEExtensions>,
    }
    impl CancelledCellsInTAINRItem {
        pub fn new(
            n_r_cgi: NRCGI,
            number_of_broadcasts: NumberOfBroadcasts,
            i_e_extensions: Option<CancelledCellsInTAINRItemIEExtensions>,
        ) -> Self {
            Self { n_r_cgi, number_of_broadcasts, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum CauseChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct CauseChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: CauseChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl CauseChoiceExtensions {
        pub fn new(id: u16, criticality: CauseChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum Cause {
        radioNetwork(CauseRadioNetwork),
        transport(CauseTransport),
        nas(CauseNas),
        protocol(CauseProtocol),
        misc(CauseMisc),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(CauseChoiceExtensions),
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum CauseMisc {
        #[rasn(identifier = "control-processing-overload")]
        control_processing_overload = 0,
        #[rasn(identifier = "not-enough-user-plane-processing-resources")]
        not_enough_user_plane_processing_resources = 1,
        #[rasn(identifier = "hardware-failure")]
        hardware_failure = 2,
        #[rasn(identifier = "om-intervention")]
        om_intervention = 3,
        #[rasn(identifier = "unknown-PLMN")]
        unknown_PLMN = 4,
        unspecified = 5,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum CauseNas {
        #[rasn(identifier = "normal-release")]
        normal_release = 0,
        #[rasn(identifier = "authentication-failure")]
        authentication_failure = 1,
        deregister = 2,
        unspecified = 3,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum CauseProtocol {
        #[rasn(identifier = "transfer-syntax-error")]
        transfer_syntax_error = 0,
        #[rasn(identifier = "abstract-syntax-error-reject")]
        abstract_syntax_error_reject = 1,
        #[rasn(identifier = "abstract-syntax-error-ignore-and-notify")]
        abstract_syntax_error_ignore_and_notify = 2,
        #[rasn(identifier = "message-not-compatible-with-receiver-state")]
        message_not_compatible_with_receiver_state = 3,
        #[rasn(identifier = "semantic-error")]
        semantic_error = 4,
        #[rasn(identifier = "abstract-syntax-error-falsely-constructed-message")]
        abstract_syntax_error_falsely_constructed_message = 5,
        unspecified = 6,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum CauseRadioNetwork {
        unspecified = 0,
        #[rasn(identifier = "txnrelocoverall-expiry")]
        txnrelocoverall_expiry = 1,
        #[rasn(identifier = "successful-handover")]
        successful_handover = 2,
        #[rasn(identifier = "release-due-to-ngran-generated-reason")]
        release_due_to_ngran_generated_reason = 3,
        #[rasn(identifier = "release-due-to-5gc-generated-reason")]
        release_due_to_5gc_generated_reason = 4,
        #[rasn(identifier = "handover-cancelled")]
        handover_cancelled = 5,
        #[rasn(identifier = "partial-handover")]
        partial_handover = 6,
        #[rasn(identifier = "ho-failure-in-target-5GC-ngran-node-or-target-system")]
        ho_failure_in_target_5GC_ngran_node_or_target_system = 7,
        #[rasn(identifier = "ho-target-not-allowed")]
        ho_target_not_allowed = 8,
        #[rasn(identifier = "tngrelocoverall-expiry")]
        tngrelocoverall_expiry = 9,
        #[rasn(identifier = "tngrelocprep-expiry")]
        tngrelocprep_expiry = 10,
        #[rasn(identifier = "cell-not-available")]
        cell_not_available = 11,
        #[rasn(identifier = "unknown-targetID")]
        unknown_targetID = 12,
        #[rasn(identifier = "no-radio-resources-available-in-target-cell")]
        no_radio_resources_available_in_target_cell = 13,
        #[rasn(identifier = "unknown-local-UE-NGAP-ID")]
        unknown_local_UE_NGAP_ID = 14,
        #[rasn(identifier = "inconsistent-remote-UE-NGAP-ID")]
        inconsistent_remote_UE_NGAP_ID = 15,
        #[rasn(identifier = "handover-desirable-for-radio-reason")]
        handover_desirable_for_radio_reason = 16,
        #[rasn(identifier = "time-critical-handover")]
        time_critical_handover = 17,
        #[rasn(identifier = "resource-optimisation-handover")]
        resource_optimisation_handover = 18,
        #[rasn(identifier = "reduce-load-in-serving-cell")]
        reduce_load_in_serving_cell = 19,
        #[rasn(identifier = "user-inactivity")]
        user_inactivity = 20,
        #[rasn(identifier = "radio-connection-with-ue-lost")]
        radio_connection_with_ue_lost = 21,
        #[rasn(identifier = "radio-resources-not-available")]
        radio_resources_not_available = 22,
        #[rasn(identifier = "invalid-qos-combination")]
        invalid_qos_combination = 23,
        #[rasn(identifier = "failure-in-radio-interface-procedure")]
        failure_in_radio_interface_procedure = 24,
        #[rasn(identifier = "interaction-with-other-procedure")]
        interaction_with_other_procedure = 25,
        #[rasn(identifier = "unknown-PDU-session-ID")]
        unknown_PDU_session_ID = 26,
        #[rasn(identifier = "unkown-qos-flow-ID")]
        unkown_qos_flow_ID = 27,
        #[rasn(identifier = "multiple-PDU-session-ID-instances")]
        multiple_PDU_session_ID_instances = 28,
        #[rasn(identifier = "multiple-qos-flow-ID-instances")]
        multiple_qos_flow_ID_instances = 29,
        #[rasn(identifier = "encryption-and-or-integrity-protection-algorithms-not-supported")]
        encryption_and_or_integrity_protection_algorithms_not_supported = 30,
        #[rasn(identifier = "ng-intra-system-handover-triggered")]
        ng_intra_system_handover_triggered = 31,
        #[rasn(identifier = "ng-inter-system-handover-triggered")]
        ng_inter_system_handover_triggered = 32,
        #[rasn(identifier = "xn-handover-triggered")]
        xn_handover_triggered = 33,
        #[rasn(identifier = "not-supported-5QI-value")]
        not_supported_5QI_value = 34,
        #[rasn(identifier = "ue-context-transfer")]
        ue_context_transfer = 35,
        #[rasn(identifier = "ims-voice-eps-fallback-or-rat-fallback-triggered")]
        ims_voice_eps_fallback_or_rat_fallback_triggered = 36,
        #[rasn(identifier = "up-integrity-protection-not-possible")]
        up_integrity_protection_not_possible = 37,
        #[rasn(identifier = "up-confidentiality-protection-not-possible")]
        up_confidentiality_protection_not_possible = 38,
        #[rasn(identifier = "slice-not-supported")]
        slice_not_supported = 39,
        #[rasn(identifier = "ue-in-rrc-inactive-state-not-reachable")]
        ue_in_rrc_inactive_state_not_reachable = 40,
        redirection = 41,
        #[rasn(identifier = "resources-not-available-for-the-slice")]
        resources_not_available_for_the_slice = 42,
        #[rasn(identifier = "ue-max-integrity-protected-data-rate-reason")]
        ue_max_integrity_protected_data_rate_reason = 43,
        #[rasn(identifier = "release-due-to-cn-detected-mobility")]
        release_due_to_cn_detected_mobility = 44,
        #[rasn(extension_addition, identifier = "n26-interface-not-available")]
        n26_interface_not_available = 45,
        #[rasn(extension_addition, identifier = "release-due-to-pre-emption")]
        release_due_to_pre_emption = 46,
        #[rasn(extension_addition, identifier = "multiple-location-reporting-reference-ID-instances")]
        multiple_location_reporting_reference_ID_instances = 47,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum CauseTransport {
        #[rasn(identifier = "transport-resource-unavailable")]
        transport_resource_unavailable = 0,
        unspecified = 1,
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CellIDBroadcastEUTRA(pub SequenceOf<CellIDBroadcastEUTRAItem>);
    aper_sequence_of!(CellIDBroadcastEUTRA, CellIDBroadcastEUTRAItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCellIDBroadcastEUTRAItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCellIDBroadcastEUTRAItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCellIDBroadcastEUTRAItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCellIDBroadcastEUTRAItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCellIDBroadcastEUTRAItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CellIDBroadcastEUTRAItemIEExtensions(pub SequenceOf<AnonymousCellIDBroadcastEUTRAItemIEExtensions>);
    aper_sequence_of!(CellIDBroadcastEUTRAItemIEExtensions, AnonymousCellIDBroadcastEUTRAItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "CellIDBroadcastEUTRA-Item")]
    #[non_exhaustive]
    pub struct CellIDBroadcastEUTRAItem {
        #[rasn(identifier = "eUTRA-CGI")]
        pub e_utra_cgi: EUTRACGI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CellIDBroadcastEUTRAItemIEExtensions>,
    }
    impl CellIDBroadcastEUTRAItem {
        pub fn new(e_utra_cgi: EUTRACGI, i_e_extensions: Option<CellIDBroadcastEUTRAItemIEExtensions>) -> Self {
            Self { e_utra_cgi, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CellIDBroadcastNR(pub SequenceOf<CellIDBroadcastNRItem>);
    aper_sequence_of!(CellIDBroadcastNR, CellIDBroadcastNRItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCellIDBroadcastNRItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCellIDBroadcastNRItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCellIDBroadcastNRItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCellIDBroadcastNRItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCellIDBroadcastNRItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CellIDBroadcastNRItemIEExtensions(pub SequenceOf<AnonymousCellIDBroadcastNRItemIEExtensions>);
    aper_sequence_of!(CellIDBroadcastNRItemIEExtensions, AnonymousCellIDBroadcastNRItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "CellIDBroadcastNR-Item")]
    #[non_exhaustive]
    pub struct CellIDBroadcastNRItem {
        #[rasn(identifier = "nR-CGI")]
        pub n_r_cgi: NRCGI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CellIDBroadcastNRItemIEExtensions>,
    }
    impl CellIDBroadcastNRItem {
        pub fn new(n_r_cgi: NRCGI, i_e_extensions: Option<CellIDBroadcastNRItemIEExtensions>) -> Self {
            Self { n_r_cgi, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CellIDCancelledEUTRA(pub SequenceOf<CellIDCancelledEUTRAItem>);
    aper_sequence_of!(CellIDCancelledEUTRA, CellIDCancelledEUTRAItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCellIDCancelledEUTRAItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCellIDCancelledEUTRAItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCellIDCancelledEUTRAItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCellIDCancelledEUTRAItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCellIDCancelledEUTRAItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CellIDCancelledEUTRAItemIEExtensions(pub SequenceOf<AnonymousCellIDCancelledEUTRAItemIEExtensions>);
    aper_sequence_of!(CellIDCancelledEUTRAItemIEExtensions, AnonymousCellIDCancelledEUTRAItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "CellIDCancelledEUTRA-Item")]
    #[non_exhaustive]
    pub struct CellIDCancelledEUTRAItem {
        #[rasn(identifier = "eUTRA-CGI")]
        pub e_utra_cgi: EUTRACGI,
        #[rasn(identifier = "numberOfBroadcasts")]
        pub number_of_broadcasts: NumberOfBroadcasts,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CellIDCancelledEUTRAItemIEExtensions>,
    }
    impl CellIDCancelledEUTRAItem {
        pub fn new(
            e_utra_cgi: EUTRACGI,
            number_of_broadcasts: NumberOfBroadcasts,
            i_e_extensions: Option<CellIDCancelledEUTRAItemIEExtensions>,
        ) -> Self {
            Self { e_utra_cgi, number_of_broadcasts, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CellIDCancelledNR(pub SequenceOf<CellIDCancelledNRItem>);
    aper_sequence_of!(CellIDCancelledNR, CellIDCancelledNRItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCellIDCancelledNRItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCellIDCancelledNRItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCellIDCancelledNRItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCellIDCancelledNRItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCellIDCancelledNRItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CellIDCancelledNRItemIEExtensions(pub SequenceOf<AnonymousCellIDCancelledNRItemIEExtensions>);
    aper_sequence_of!(CellIDCancelledNRItemIEExtensions, AnonymousCellIDCancelledNRItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "CellIDCancelledNR-Item")]
    #[non_exhaustive]
    pub struct CellIDCancelledNRItem {
        #[rasn(identifier = "nR-CGI")]
        pub n_r_cgi: NRCGI,
        #[rasn(identifier = "numberOfBroadcasts")]
        pub number_of_broadcasts: NumberOfBroadcasts,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CellIDCancelledNRItemIEExtensions>,
    }
    impl CellIDCancelledNRItem {
        pub fn new(
            n_r_cgi: NRCGI,
            number_of_broadcasts: NumberOfBroadcasts,
            i_e_extensions: Option<CellIDCancelledNRItemIEExtensions>,
        ) -> Self {
            Self { n_r_cgi, number_of_broadcasts, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum CellIDListForRestartChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct CellIDListForRestartChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: CellIDListForRestartChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl CellIDListForRestartChoiceExtensions {
        pub fn new(id: u16, criticality: CellIDListForRestartChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum CellIDListForRestart {
        #[rasn(identifier = "eUTRA-CGIListforRestart")]
        eUTRA_CGIListforRestart(EUTRACGIList),
        #[rasn(identifier = "nR-CGIListforRestart")]
        nR_CGIListforRestart(NRCGIList),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(CellIDListForRestartChoiceExtensions),
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum CellSize {
        verysmall = 0,
        small = 1,
        medium = 2,
        large = 3,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCellTypeIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCellTypeIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCellTypeIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCellTypeIEExtensions {
        pub fn new(id: u16, criticality: AnonymousCellTypeIEExtensionsCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CellTypeIEExtensions(pub SequenceOf<AnonymousCellTypeIEExtensions>);
    aper_sequence_of!(CellTypeIEExtensions, AnonymousCellTypeIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct CellType {
        #[rasn(identifier = "cellSize")]
        pub cell_size: CellSize,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CellTypeIEExtensions>,
    }
    impl CellType {
        pub fn new(cell_size: CellSize, i_e_extensions: Option<CellTypeIEExtensions>) -> Self {
            Self { cell_size, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct CommonNetworkInstance(pub OctetString);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"), identifier = "CompletedCellsInEAI-EUTRA")]
    pub struct CompletedCellsInEAIEUTRA(pub SequenceOf<CompletedCellsInEAIEUTRAItem>);
    aper_sequence_of!(CompletedCellsInEAIEUTRA, CompletedCellsInEAIEUTRAItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCompletedCellsInEAIEUTRAItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCompletedCellsInEAIEUTRAItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCompletedCellsInEAIEUTRAItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCompletedCellsInEAIEUTRAItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCompletedCellsInEAIEUTRAItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CompletedCellsInEAIEUTRAItemIEExtensions(
        pub SequenceOf<AnonymousCompletedCellsInEAIEUTRAItemIEExtensions>,
    );
    aper_sequence_of!(
        CompletedCellsInEAIEUTRAItemIEExtensions,
        AnonymousCompletedCellsInEAIEUTRAItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "CompletedCellsInEAI-EUTRA-Item")]
    #[non_exhaustive]
    pub struct CompletedCellsInEAIEUTRAItem {
        #[rasn(identifier = "eUTRA-CGI")]
        pub e_utra_cgi: EUTRACGI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CompletedCellsInEAIEUTRAItemIEExtensions>,
    }
    impl CompletedCellsInEAIEUTRAItem {
        pub fn new(e_utra_cgi: EUTRACGI, i_e_extensions: Option<CompletedCellsInEAIEUTRAItemIEExtensions>) -> Self {
            Self { e_utra_cgi, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"), identifier = "CompletedCellsInEAI-NR")]
    pub struct CompletedCellsInEAINR(pub SequenceOf<CompletedCellsInEAINRItem>);
    aper_sequence_of!(CompletedCellsInEAINR, CompletedCellsInEAINRItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCompletedCellsInEAINRItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCompletedCellsInEAINRItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCompletedCellsInEAINRItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCompletedCellsInEAINRItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCompletedCellsInEAINRItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CompletedCellsInEAINRItemIEExtensions(pub SequenceOf<AnonymousCompletedCellsInEAINRItemIEExtensions>);
    aper_sequence_of!(CompletedCellsInEAINRItemIEExtensions, AnonymousCompletedCellsInEAINRItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "CompletedCellsInEAI-NR-Item")]
    #[non_exhaustive]
    pub struct CompletedCellsInEAINRItem {
        #[rasn(identifier = "nR-CGI")]
        pub n_r_cgi: NRCGI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CompletedCellsInEAINRItemIEExtensions>,
    }
    impl CompletedCellsInEAINRItem {
        pub fn new(n_r_cgi: NRCGI, i_e_extensions: Option<CompletedCellsInEAINRItemIEExtensions>) -> Self {
            Self { n_r_cgi, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"), identifier = "CompletedCellsInTAI-EUTRA")]
    pub struct CompletedCellsInTAIEUTRA(pub SequenceOf<CompletedCellsInTAIEUTRAItem>);
    aper_sequence_of!(CompletedCellsInTAIEUTRA, CompletedCellsInTAIEUTRAItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCompletedCellsInTAIEUTRAItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCompletedCellsInTAIEUTRAItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCompletedCellsInTAIEUTRAItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCompletedCellsInTAIEUTRAItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCompletedCellsInTAIEUTRAItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CompletedCellsInTAIEUTRAItemIEExtensions(
        pub SequenceOf<AnonymousCompletedCellsInTAIEUTRAItemIEExtensions>,
    );
    aper_sequence_of!(
        CompletedCellsInTAIEUTRAItemIEExtensions,
        AnonymousCompletedCellsInTAIEUTRAItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "CompletedCellsInTAI-EUTRA-Item")]
    #[non_exhaustive]
    pub struct CompletedCellsInTAIEUTRAItem {
        #[rasn(identifier = "eUTRA-CGI")]
        pub e_utra_cgi: EUTRACGI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CompletedCellsInTAIEUTRAItemIEExtensions>,
    }
    impl CompletedCellsInTAIEUTRAItem {
        pub fn new(e_utra_cgi: EUTRACGI, i_e_extensions: Option<CompletedCellsInTAIEUTRAItemIEExtensions>) -> Self {
            Self { e_utra_cgi, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"), identifier = "CompletedCellsInTAI-NR")]
    pub struct CompletedCellsInTAINR(pub SequenceOf<CompletedCellsInTAINRItem>);
    aper_sequence_of!(CompletedCellsInTAINR, CompletedCellsInTAINRItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCompletedCellsInTAINRItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCompletedCellsInTAINRItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCompletedCellsInTAINRItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCompletedCellsInTAINRItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCompletedCellsInTAINRItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CompletedCellsInTAINRItemIEExtensions(pub SequenceOf<AnonymousCompletedCellsInTAINRItemIEExtensions>);
    aper_sequence_of!(CompletedCellsInTAINRItemIEExtensions, AnonymousCompletedCellsInTAINRItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "CompletedCellsInTAI-NR-Item")]
    #[non_exhaustive]
    pub struct CompletedCellsInTAINRItem {
        #[rasn(identifier = "nR-CGI")]
        pub n_r_cgi: NRCGI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CompletedCellsInTAINRItemIEExtensions>,
    }
    impl CompletedCellsInTAINRItem {
        pub fn new(n_r_cgi: NRCGI, i_e_extensions: Option<CompletedCellsInTAINRItemIEExtensions>) -> Self {
            Self { n_r_cgi, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum ConcurrentWarningMessageInd {
        #[rasn(identifier = "true")]
        R_true = 0,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum ConfidentialityProtectionIndication {
        required = 0,
        preferred = 1,
        #[rasn(identifier = "not-needed")]
        not_needed = 2,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum ConfidentialityProtectionResult {
        performed = 0,
        #[rasn(identifier = "not-performed")]
        not_performed = 1,
    }
    #[doc = " This IE includes information from the source Core node to the target Core node for reroute information provide by NSSF."]
    #[doc = " The octets of the OCTET STRING are encoded according to the specifications of the Core network."]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct ConfiguredNSSAI(pub FixedOctetString<128usize>);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCoreNetworkAssistanceInformationForInactiveIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCoreNetworkAssistanceInformationForInactiveIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCoreNetworkAssistanceInformationForInactiveIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCoreNetworkAssistanceInformationForInactiveIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCoreNetworkAssistanceInformationForInactiveIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CoreNetworkAssistanceInformationForInactiveIEExtensions(
        pub SequenceOf<AnonymousCoreNetworkAssistanceInformationForInactiveIEExtensions>,
    );
    aper_sequence_of!(
        CoreNetworkAssistanceInformationForInactiveIEExtensions,
        AnonymousCoreNetworkAssistanceInformationForInactiveIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct CoreNetworkAssistanceInformationForInactive {
        #[rasn(identifier = "uEIdentityIndexValue")]
        pub u_eidentity_index_value: UEIdentityIndexValue,
        #[rasn(identifier = "uESpecificDRX")]
        pub u_especific_drx: Option<PagingDRX>,
        #[rasn(identifier = "periodicRegistrationUpdateTimer")]
        pub periodic_registration_update_timer: PeriodicRegistrationUpdateTimer,
        #[rasn(identifier = "mICOModeIndication")]
        pub m_icomode_indication: Option<MICOModeIndication>,
        #[rasn(identifier = "tAIListForInactive")]
        pub t_ailist_for_inactive: TAIListForInactive,
        #[rasn(identifier = "expectedUEBehaviour")]
        pub expected_uebehaviour: Option<ExpectedUEBehaviour>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CoreNetworkAssistanceInformationForInactiveIEExtensions>,
    }
    impl CoreNetworkAssistanceInformationForInactive {
        pub fn new(
            u_eidentity_index_value: UEIdentityIndexValue,
            u_especific_drx: Option<PagingDRX>,
            periodic_registration_update_timer: PeriodicRegistrationUpdateTimer,
            m_icomode_indication: Option<MICOModeIndication>,
            t_ailist_for_inactive: TAIListForInactive,
            expected_uebehaviour: Option<ExpectedUEBehaviour>,
            i_e_extensions: Option<CoreNetworkAssistanceInformationForInactiveIEExtensions>,
        ) -> Self {
            Self {
                u_eidentity_index_value,
                u_especific_drx,
                periodic_registration_update_timer,
                m_icomode_indication,
                t_ailist_for_inactive,
                expected_uebehaviour,
                i_e_extensions,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCriticalityDiagnosticsIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCriticalityDiagnosticsIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCriticalityDiagnosticsIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCriticalityDiagnosticsIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCriticalityDiagnosticsIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CriticalityDiagnosticsIEExtensions(pub SequenceOf<AnonymousCriticalityDiagnosticsIEExtensions>);
    aper_sequence_of!(CriticalityDiagnosticsIEExtensions, AnonymousCriticalityDiagnosticsIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct CriticalityDiagnostics {
        #[rasn(identifier = "procedureCode")]
        pub procedure_code: Option<ProcedureCode>,
        #[rasn(identifier = "triggeringMessage")]
        pub triggering_message: Option<TriggeringMessage>,
        #[rasn(identifier = "procedureCriticality")]
        pub procedure_criticality: Option<Criticality>,
        #[rasn(identifier = "iEsCriticalityDiagnostics")]
        pub i_es_criticality_diagnostics: Option<CriticalityDiagnosticsIEList>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CriticalityDiagnosticsIEExtensions>,
    }
    impl CriticalityDiagnostics {
        pub fn new(
            procedure_code: Option<ProcedureCode>,
            triggering_message: Option<TriggeringMessage>,
            procedure_criticality: Option<Criticality>,
            i_es_criticality_diagnostics: Option<CriticalityDiagnosticsIEList>,
            i_e_extensions: Option<CriticalityDiagnosticsIEExtensions>,
        ) -> Self {
            Self {
                procedure_code,
                triggering_message,
                procedure_criticality,
                i_es_criticality_diagnostics,
                i_e_extensions,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCriticalityDiagnosticsIEItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCriticalityDiagnosticsIEItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCriticalityDiagnosticsIEItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousCriticalityDiagnosticsIEItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousCriticalityDiagnosticsIEItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct CriticalityDiagnosticsIEItemIEExtensions(
        pub SequenceOf<AnonymousCriticalityDiagnosticsIEItemIEExtensions>,
    );
    aper_sequence_of!(
        CriticalityDiagnosticsIEItemIEExtensions,
        AnonymousCriticalityDiagnosticsIEItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "CriticalityDiagnostics-IE-Item")]
    #[non_exhaustive]
    pub struct CriticalityDiagnosticsIEItem {
        #[rasn(identifier = "iECriticality")]
        pub i_ecriticality: Criticality,
        #[rasn(identifier = "iE-ID")]
        pub i_e_id: ProtocolIEID,
        #[rasn(identifier = "typeOfError")]
        pub type_of_error: TypeOfError,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<CriticalityDiagnosticsIEItemIEExtensions>,
    }
    impl CriticalityDiagnosticsIEItem {
        pub fn new(
            i_ecriticality: Criticality,
            i_e_id: ProtocolIEID,
            type_of_error: TypeOfError,
            i_e_extensions: Option<CriticalityDiagnosticsIEItemIEExtensions>,
        ) -> Self {
            Self { i_ecriticality, i_e_id, type_of_error, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"), identifier = "CriticalityDiagnostics-IE-List")]
    pub struct CriticalityDiagnosticsIEList(pub SequenceOf<CriticalityDiagnosticsIEItem>);
    aper_sequence_of!(CriticalityDiagnosticsIEList, CriticalityDiagnosticsIEItem, 1, 256);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated, identifier = "DL-NGU-TNLInformationReused")]
    #[non_exhaustive]
    pub enum DLNGUTNLInformationReused {
        #[rasn(identifier = "true")]
        R_true = 0,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum DLForwarding {
        #[rasn(identifier = "dl-forwarding-proposed")]
        dl_forwarding_proposed = 0,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "DRB-ID", value("1..=32", extensible))]
    pub struct DRBID(pub Integer);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum DRBStatusDLChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct DRBStatusDLChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: DRBStatusDLChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl DRBStatusDLChoiceExtensions {
        pub fn new(id: u16, criticality: DRBStatusDLChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum DRBStatusDL {
        dRBStatusDL12(DRBStatusDL12),
        dRBStatusDL18(DRBStatusDL18),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(DRBStatusDLChoiceExtensions),
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDRBStatusDL12IEExtensionCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDRBStatusDL12IEExtension {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDRBStatusDL12IEExtensionCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousDRBStatusDL12IEExtension {
        pub fn new(id: u16, criticality: AnonymousDRBStatusDL12IEExtensionCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct DRBStatusDL12IEExtension(pub SequenceOf<AnonymousDRBStatusDL12IEExtension>);
    aper_sequence_of!(DRBStatusDL12IEExtension, AnonymousDRBStatusDL12IEExtension, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct DRBStatusDL12 {
        #[rasn(identifier = "dL-COUNTValue")]
        pub d_l_countvalue: COUNTValueForPDCPSN12,
        #[rasn(identifier = "iE-Extension")]
        pub i_e_extension: Option<DRBStatusDL12IEExtension>,
    }
    impl DRBStatusDL12 {
        pub fn new(d_l_countvalue: COUNTValueForPDCPSN12, i_e_extension: Option<DRBStatusDL12IEExtension>) -> Self {
            Self { d_l_countvalue, i_e_extension }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDRBStatusDL18IEExtensionCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDRBStatusDL18IEExtension {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDRBStatusDL18IEExtensionCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousDRBStatusDL18IEExtension {
        pub fn new(id: u16, criticality: AnonymousDRBStatusDL18IEExtensionCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct DRBStatusDL18IEExtension(pub SequenceOf<AnonymousDRBStatusDL18IEExtension>);
    aper_sequence_of!(DRBStatusDL18IEExtension, AnonymousDRBStatusDL18IEExtension, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct DRBStatusDL18 {
        #[rasn(identifier = "dL-COUNTValue")]
        pub d_l_countvalue: COUNTValueForPDCPSN18,
        #[rasn(identifier = "iE-Extension")]
        pub i_e_extension: Option<DRBStatusDL18IEExtension>,
    }
    impl DRBStatusDL18 {
        pub fn new(d_l_countvalue: COUNTValueForPDCPSN18, i_e_extension: Option<DRBStatusDL18IEExtension>) -> Self {
            Self { d_l_countvalue, i_e_extension }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum DRBStatusULChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct DRBStatusULChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: DRBStatusULChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl DRBStatusULChoiceExtensions {
        pub fn new(id: u16, criticality: DRBStatusULChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum DRBStatusUL {
        dRBStatusUL12(DRBStatusUL12),
        dRBStatusUL18(DRBStatusUL18),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(DRBStatusULChoiceExtensions),
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDRBStatusUL12IEExtensionCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDRBStatusUL12IEExtension {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDRBStatusUL12IEExtensionCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousDRBStatusUL12IEExtension {
        pub fn new(id: u16, criticality: AnonymousDRBStatusUL12IEExtensionCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct DRBStatusUL12IEExtension(pub SequenceOf<AnonymousDRBStatusUL12IEExtension>);
    aper_sequence_of!(DRBStatusUL12IEExtension, AnonymousDRBStatusUL12IEExtension, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct DRBStatusUL12 {
        #[rasn(identifier = "uL-COUNTValue")]
        pub u_l_countvalue: COUNTValueForPDCPSN12,
        #[rasn(size("1..=2048"), identifier = "receiveStatusOfUL-PDCP-SDUs")]
        pub receive_status_of_ul_pdcp_sdus: Option<BitString>,
        #[rasn(identifier = "iE-Extension")]
        pub i_e_extension: Option<DRBStatusUL12IEExtension>,
    }
    impl DRBStatusUL12 {
        pub fn new(
            u_l_countvalue: COUNTValueForPDCPSN12,
            receive_status_of_ul_pdcp_sdus: Option<BitString>,
            i_e_extension: Option<DRBStatusUL12IEExtension>,
        ) -> Self {
            Self { u_l_countvalue, receive_status_of_ul_pdcp_sdus, i_e_extension }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDRBStatusUL18IEExtensionCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDRBStatusUL18IEExtension {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDRBStatusUL18IEExtensionCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousDRBStatusUL18IEExtension {
        pub fn new(id: u16, criticality: AnonymousDRBStatusUL18IEExtensionCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct DRBStatusUL18IEExtension(pub SequenceOf<AnonymousDRBStatusUL18IEExtension>);
    aper_sequence_of!(DRBStatusUL18IEExtension, AnonymousDRBStatusUL18IEExtension, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct DRBStatusUL18 {
        #[rasn(identifier = "uL-COUNTValue")]
        pub u_l_countvalue: COUNTValueForPDCPSN18,
        #[rasn(size("1..=131072"), identifier = "receiveStatusOfUL-PDCP-SDUs")]
        pub receive_status_of_ul_pdcp_sdus: Option<BitString>,
        #[rasn(identifier = "iE-Extension")]
        pub i_e_extension: Option<DRBStatusUL18IEExtension>,
    }
    impl DRBStatusUL18 {
        pub fn new(
            u_l_countvalue: COUNTValueForPDCPSN18,
            receive_status_of_ul_pdcp_sdus: Option<BitString>,
            i_e_extension: Option<DRBStatusUL18IEExtension>,
        ) -> Self {
            Self { u_l_countvalue, receive_status_of_ul_pdcp_sdus, i_e_extension }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDRBsSubjectToStatusTransferItemIEExtensionCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDRBsSubjectToStatusTransferItemIEExtension {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDRBsSubjectToStatusTransferItemIEExtensionCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousDRBsSubjectToStatusTransferItemIEExtension {
        pub fn new(
            id: u16,
            criticality: AnonymousDRBsSubjectToStatusTransferItemIEExtensionCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct DRBsSubjectToStatusTransferItemIEExtension(
        pub SequenceOf<AnonymousDRBsSubjectToStatusTransferItemIEExtension>,
    );
    aper_sequence_of!(
        DRBsSubjectToStatusTransferItemIEExtension,
        AnonymousDRBsSubjectToStatusTransferItemIEExtension,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct DRBsSubjectToStatusTransferItem {
        #[rasn(identifier = "dRB-ID")]
        pub d_rb_id: DRBID,
        #[rasn(identifier = "dRBStatusUL")]
        pub d_rbstatus_ul: DRBStatusUL,
        #[rasn(identifier = "dRBStatusDL")]
        pub d_rbstatus_dl: DRBStatusDL,
        #[rasn(identifier = "iE-Extension")]
        pub i_e_extension: Option<DRBsSubjectToStatusTransferItemIEExtension>,
    }
    impl DRBsSubjectToStatusTransferItem {
        pub fn new(
            d_rb_id: DRBID,
            d_rbstatus_ul: DRBStatusUL,
            d_rbstatus_dl: DRBStatusDL,
            i_e_extension: Option<DRBsSubjectToStatusTransferItemIEExtension>,
        ) -> Self {
            Self { d_rb_id, d_rbstatus_ul, d_rbstatus_dl, i_e_extension }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=32"))]
    pub struct DRBsSubjectToStatusTransferList(pub SequenceOf<DRBsSubjectToStatusTransferItem>);
    aper_sequence_of!(DRBsSubjectToStatusTransferList, DRBsSubjectToStatusTransferItem, 1, 32);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDRBsToQosFlowsMappingItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDRBsToQosFlowsMappingItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDRBsToQosFlowsMappingItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousDRBsToQosFlowsMappingItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousDRBsToQosFlowsMappingItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct DRBsToQosFlowsMappingItemIEExtensions(pub SequenceOf<AnonymousDRBsToQosFlowsMappingItemIEExtensions>);
    aper_sequence_of!(DRBsToQosFlowsMappingItemIEExtensions, AnonymousDRBsToQosFlowsMappingItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct DRBsToQosFlowsMappingItem {
        #[rasn(identifier = "dRB-ID")]
        pub d_rb_id: DRBID,
        #[rasn(identifier = "associatedQosFlowList")]
        pub associated_qos_flow_list: AssociatedQosFlowList,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<DRBsToQosFlowsMappingItemIEExtensions>,
    }
    impl DRBsToQosFlowsMappingItem {
        pub fn new(
            d_rb_id: DRBID,
            associated_qos_flow_list: AssociatedQosFlowList,
            i_e_extensions: Option<DRBsToQosFlowsMappingItemIEExtensions>,
        ) -> Self {
            Self { d_rb_id, associated_qos_flow_list, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=32"))]
    pub struct DRBsToQosFlowsMappingList(pub SequenceOf<DRBsToQosFlowsMappingItem>);
    aper_sequence_of!(DRBsToQosFlowsMappingList, DRBsToQosFlowsMappingItem, 1, 32);
    #[doc = " D"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct DataCodingScheme(pub FixedBitString<8usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum DataForwardingAccepted {
        #[rasn(identifier = "data-forwarding-accepted")]
        data_forwarding_accepted = 0,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum DataForwardingNotPossible {
        #[rasn(identifier = "data-forwarding-not-possible")]
        data_forwarding_not_possible = 0,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDataForwardingResponseDRBItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDataForwardingResponseDRBItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDataForwardingResponseDRBItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousDataForwardingResponseDRBItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousDataForwardingResponseDRBItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct DataForwardingResponseDRBItemIEExtensions(
        pub SequenceOf<AnonymousDataForwardingResponseDRBItemIEExtensions>,
    );
    aper_sequence_of!(
        DataForwardingResponseDRBItemIEExtensions,
        AnonymousDataForwardingResponseDRBItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct DataForwardingResponseDRBItem {
        #[rasn(identifier = "dRB-ID")]
        pub d_rb_id: DRBID,
        #[rasn(identifier = "dLForwardingUP-TNLInformation")]
        pub d_lforwarding_up_tnlinformation: Option<UPTransportLayerInformation>,
        #[rasn(identifier = "uLForwardingUP-TNLInformation")]
        pub u_lforwarding_up_tnlinformation: Option<UPTransportLayerInformation>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<DataForwardingResponseDRBItemIEExtensions>,
    }
    impl DataForwardingResponseDRBItem {
        pub fn new(
            d_rb_id: DRBID,
            d_lforwarding_up_tnlinformation: Option<UPTransportLayerInformation>,
            u_lforwarding_up_tnlinformation: Option<UPTransportLayerInformation>,
            i_e_extensions: Option<DataForwardingResponseDRBItemIEExtensions>,
        ) -> Self {
            Self { d_rb_id, d_lforwarding_up_tnlinformation, u_lforwarding_up_tnlinformation, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=32"))]
    pub struct DataForwardingResponseDRBList(pub SequenceOf<DataForwardingResponseDRBItem>);
    aper_sequence_of!(DataForwardingResponseDRBList, DataForwardingResponseDRBItem, 1, 32);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum DelayCritical {
        #[rasn(identifier = "delay-critical")]
        delay_critical = 0,
        #[rasn(identifier = "non-delay-critical")]
        non_delay_critical = 1,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum DirectForwardingPathAvailability {
        #[rasn(identifier = "direct-path-available")]
        direct_path_available = 0,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDynamic5QIDescriptorIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDynamic5QIDescriptorIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDynamic5QIDescriptorIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousDynamic5QIDescriptorIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousDynamic5QIDescriptorIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct Dynamic5QIDescriptorIEExtensions(pub SequenceOf<AnonymousDynamic5QIDescriptorIEExtensions>);
    aper_sequence_of!(Dynamic5QIDescriptorIEExtensions, AnonymousDynamic5QIDescriptorIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct Dynamic5QIDescriptor {
        #[rasn(identifier = "priorityLevelQos")]
        pub priority_level_qos: PriorityLevelQos,
        #[rasn(identifier = "packetDelayBudget")]
        pub packet_delay_budget: PacketDelayBudget,
        #[rasn(identifier = "packetErrorRate")]
        pub packet_error_rate: PacketErrorRate,
        #[rasn(identifier = "fiveQI")]
        pub five_qi: Option<FiveQI>,
        #[rasn(identifier = "delayCritical")]
        pub delay_critical: Option<DelayCritical>,
        #[rasn(identifier = "averagingWindow")]
        pub averaging_window: Option<AveragingWindow>,
        #[rasn(identifier = "maximumDataBurstVolume")]
        pub maximum_data_burst_volume: Option<MaximumDataBurstVolume>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<Dynamic5QIDescriptorIEExtensions>,
    }
    impl Dynamic5QIDescriptor {
        pub fn new(
            priority_level_qos: PriorityLevelQos,
            packet_delay_budget: PacketDelayBudget,
            packet_error_rate: PacketErrorRate,
            five_qi: Option<FiveQI>,
            delay_critical: Option<DelayCritical>,
            averaging_window: Option<AveragingWindow>,
            maximum_data_burst_volume: Option<MaximumDataBurstVolume>,
            i_e_extensions: Option<Dynamic5QIDescriptorIEExtensions>,
        ) -> Self {
            Self {
                priority_level_qos,
                packet_delay_budget,
                packet_error_rate,
                five_qi,
                delay_critical,
                averaging_window,
                maximum_data_burst_volume,
                i_e_extensions,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "E-RAB-ID", value("0..=15", extensible))]
    pub struct ERABID(pub Integer);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousERABInformationItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousERABInformationItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousERABInformationItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousERABInformationItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousERABInformationItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct ERABInformationItemIEExtensions(pub SequenceOf<AnonymousERABInformationItemIEExtensions>);
    aper_sequence_of!(ERABInformationItemIEExtensions, AnonymousERABInformationItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "E-RABInformationItem")]
    #[non_exhaustive]
    pub struct ERABInformationItem {
        #[rasn(identifier = "e-RAB-ID")]
        pub e_rab_id: ERABID,
        #[rasn(identifier = "dLForwarding")]
        pub d_lforwarding: Option<DLForwarding>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<ERABInformationItemIEExtensions>,
    }
    impl ERABInformationItem {
        pub fn new(
            e_rab_id: ERABID,
            d_lforwarding: Option<DLForwarding>,
            i_e_extensions: Option<ERABInformationItemIEExtensions>,
        ) -> Self {
            Self { e_rab_id, d_lforwarding, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"), identifier = "E-RABInformationList")]
    pub struct ERABInformationList(pub SequenceOf<ERABInformationItem>);
    aper_sequence_of!(ERABInformationList, ERABInformationItem, 1, 256);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "EN-DCSONConfigurationTransfer")]
    pub struct ENDCSONConfigurationTransfer(pub OctetString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "EPS-TAC")]
    pub struct EPSTAC(pub FixedOctetString<2usize>);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousEPSTAIIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousEPSTAIIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousEPSTAIIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousEPSTAIIEExtensions {
        pub fn new(id: u16, criticality: AnonymousEPSTAIIEExtensionsCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct EPSTAIIEExtensions(pub SequenceOf<AnonymousEPSTAIIEExtensions>);
    aper_sequence_of!(EPSTAIIEExtensions, AnonymousEPSTAIIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "EPS-TAI")]
    #[non_exhaustive]
    pub struct EPSTAI {
        #[rasn(identifier = "pLMNIdentity")]
        pub p_lmnidentity: PLMNIdentity,
        #[rasn(identifier = "ePS-TAC")]
        pub e_ps_tac: EPSTAC,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<EPSTAIIEExtensions>,
    }
    impl EPSTAI {
        pub fn new(p_lmnidentity: PLMNIdentity, e_ps_tac: EPSTAC, i_e_extensions: Option<EPSTAIIEExtensions>) -> Self {
            Self { p_lmnidentity, e_ps_tac, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousEUTRACGIIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousEUTRACGIIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousEUTRACGIIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousEUTRACGIIEExtensions {
        pub fn new(id: u16, criticality: AnonymousEUTRACGIIEExtensionsCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct EUTRACGIIEExtensions(pub SequenceOf<AnonymousEUTRACGIIEExtensions>);
    aper_sequence_of!(EUTRACGIIEExtensions, AnonymousEUTRACGIIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "EUTRA-CGI")]
    #[non_exhaustive]
    pub struct EUTRACGI {
        #[rasn(identifier = "pLMNIdentity")]
        pub p_lmnidentity: PLMNIdentity,
        #[rasn(identifier = "eUTRACellIdentity")]
        pub e_utracell_identity: EUTRACellIdentity,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<EUTRACGIIEExtensions>,
    }
    impl EUTRACGI {
        pub fn new(
            p_lmnidentity: PLMNIdentity,
            e_utracell_identity: EUTRACellIdentity,
            i_e_extensions: Option<EUTRACGIIEExtensions>,
        ) -> Self {
            Self { p_lmnidentity, e_utracell_identity, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"), identifier = "EUTRA-CGIList")]
    pub struct EUTRACGIList(pub SequenceOf<EUTRACGI>);
    aper_sequence_of!(EUTRACGIList, EUTRACGI, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"), identifier = "EUTRA-CGIListForWarning")]
    pub struct EUTRACGIListForWarning(pub SequenceOf<EUTRACGI>);
    aper_sequence_of!(EUTRACGIListForWarning, EUTRACGI, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct EUTRACellIdentity(pub FixedBitString<28usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("16", extensible))]
    pub struct EUTRAencryptionAlgorithms(pub BitString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("16", extensible))]
    pub struct EUTRAintegrityProtectionAlgorithms(pub BitString);
    #[doc = " E"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct EmergencyAreaID(pub FixedOctetString<3usize>);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct EmergencyAreaIDBroadcastEUTRA(pub SequenceOf<EmergencyAreaIDBroadcastEUTRAItem>);
    aper_sequence_of!(EmergencyAreaIDBroadcastEUTRA, EmergencyAreaIDBroadcastEUTRAItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousEmergencyAreaIDBroadcastEUTRAItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousEmergencyAreaIDBroadcastEUTRAItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousEmergencyAreaIDBroadcastEUTRAItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousEmergencyAreaIDBroadcastEUTRAItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousEmergencyAreaIDBroadcastEUTRAItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct EmergencyAreaIDBroadcastEUTRAItemIEExtensions(
        pub SequenceOf<AnonymousEmergencyAreaIDBroadcastEUTRAItemIEExtensions>,
    );
    aper_sequence_of!(
        EmergencyAreaIDBroadcastEUTRAItemIEExtensions,
        AnonymousEmergencyAreaIDBroadcastEUTRAItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "EmergencyAreaIDBroadcastEUTRA-Item")]
    #[non_exhaustive]
    pub struct EmergencyAreaIDBroadcastEUTRAItem {
        #[rasn(identifier = "emergencyAreaID")]
        pub emergency_area_id: EmergencyAreaID,
        #[rasn(identifier = "completedCellsInEAI-EUTRA")]
        pub completed_cells_in_eai_eutra: CompletedCellsInEAIEUTRA,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<EmergencyAreaIDBroadcastEUTRAItemIEExtensions>,
    }
    impl EmergencyAreaIDBroadcastEUTRAItem {
        pub fn new(
            emergency_area_id: EmergencyAreaID,
            completed_cells_in_eai_eutra: CompletedCellsInEAIEUTRA,
            i_e_extensions: Option<EmergencyAreaIDBroadcastEUTRAItemIEExtensions>,
        ) -> Self {
            Self { emergency_area_id, completed_cells_in_eai_eutra, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct EmergencyAreaIDBroadcastNR(pub SequenceOf<EmergencyAreaIDBroadcastNRItem>);
    aper_sequence_of!(EmergencyAreaIDBroadcastNR, EmergencyAreaIDBroadcastNRItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousEmergencyAreaIDBroadcastNRItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousEmergencyAreaIDBroadcastNRItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousEmergencyAreaIDBroadcastNRItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousEmergencyAreaIDBroadcastNRItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousEmergencyAreaIDBroadcastNRItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct EmergencyAreaIDBroadcastNRItemIEExtensions(
        pub SequenceOf<AnonymousEmergencyAreaIDBroadcastNRItemIEExtensions>,
    );
    aper_sequence_of!(
        EmergencyAreaIDBroadcastNRItemIEExtensions,
        AnonymousEmergencyAreaIDBroadcastNRItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "EmergencyAreaIDBroadcastNR-Item")]
    #[non_exhaustive]
    pub struct EmergencyAreaIDBroadcastNRItem {
        #[rasn(identifier = "emergencyAreaID")]
        pub emergency_area_id: EmergencyAreaID,
        #[rasn(identifier = "completedCellsInEAI-NR")]
        pub completed_cells_in_eai_nr: CompletedCellsInEAINR,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<EmergencyAreaIDBroadcastNRItemIEExtensions>,
    }
    impl EmergencyAreaIDBroadcastNRItem {
        pub fn new(
            emergency_area_id: EmergencyAreaID,
            completed_cells_in_eai_nr: CompletedCellsInEAINR,
            i_e_extensions: Option<EmergencyAreaIDBroadcastNRItemIEExtensions>,
        ) -> Self {
            Self { emergency_area_id, completed_cells_in_eai_nr, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct EmergencyAreaIDCancelledEUTRA(pub SequenceOf<EmergencyAreaIDCancelledEUTRAItem>);
    aper_sequence_of!(EmergencyAreaIDCancelledEUTRA, EmergencyAreaIDCancelledEUTRAItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousEmergencyAreaIDCancelledEUTRAItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousEmergencyAreaIDCancelledEUTRAItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousEmergencyAreaIDCancelledEUTRAItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousEmergencyAreaIDCancelledEUTRAItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousEmergencyAreaIDCancelledEUTRAItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct EmergencyAreaIDCancelledEUTRAItemIEExtensions(
        pub SequenceOf<AnonymousEmergencyAreaIDCancelledEUTRAItemIEExtensions>,
    );
    aper_sequence_of!(
        EmergencyAreaIDCancelledEUTRAItemIEExtensions,
        AnonymousEmergencyAreaIDCancelledEUTRAItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "EmergencyAreaIDCancelledEUTRA-Item")]
    #[non_exhaustive]
    pub struct EmergencyAreaIDCancelledEUTRAItem {
        #[rasn(identifier = "emergencyAreaID")]
        pub emergency_area_id: EmergencyAreaID,
        #[rasn(identifier = "cancelledCellsInEAI-EUTRA")]
        pub cancelled_cells_in_eai_eutra: CancelledCellsInEAIEUTRA,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<EmergencyAreaIDCancelledEUTRAItemIEExtensions>,
    }
    impl EmergencyAreaIDCancelledEUTRAItem {
        pub fn new(
            emergency_area_id: EmergencyAreaID,
            cancelled_cells_in_eai_eutra: CancelledCellsInEAIEUTRA,
            i_e_extensions: Option<EmergencyAreaIDCancelledEUTRAItemIEExtensions>,
        ) -> Self {
            Self { emergency_area_id, cancelled_cells_in_eai_eutra, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct EmergencyAreaIDCancelledNR(pub SequenceOf<EmergencyAreaIDCancelledNRItem>);
    aper_sequence_of!(EmergencyAreaIDCancelledNR, EmergencyAreaIDCancelledNRItem, 1, 65535);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousEmergencyAreaIDCancelledNRItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousEmergencyAreaIDCancelledNRItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousEmergencyAreaIDCancelledNRItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousEmergencyAreaIDCancelledNRItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousEmergencyAreaIDCancelledNRItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct EmergencyAreaIDCancelledNRItemIEExtensions(
        pub SequenceOf<AnonymousEmergencyAreaIDCancelledNRItemIEExtensions>,
    );
    aper_sequence_of!(
        EmergencyAreaIDCancelledNRItemIEExtensions,
        AnonymousEmergencyAreaIDCancelledNRItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "EmergencyAreaIDCancelledNR-Item")]
    #[non_exhaustive]
    pub struct EmergencyAreaIDCancelledNRItem {
        #[rasn(identifier = "emergencyAreaID")]
        pub emergency_area_id: EmergencyAreaID,
        #[rasn(identifier = "cancelledCellsInEAI-NR")]
        pub cancelled_cells_in_eai_nr: CancelledCellsInEAINR,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<EmergencyAreaIDCancelledNRItemIEExtensions>,
    }
    impl EmergencyAreaIDCancelledNRItem {
        pub fn new(
            emergency_area_id: EmergencyAreaID,
            cancelled_cells_in_eai_nr: CancelledCellsInEAINR,
            i_e_extensions: Option<EmergencyAreaIDCancelledNRItemIEExtensions>,
        ) -> Self {
            Self { emergency_area_id, cancelled_cells_in_eai_nr, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct EmergencyAreaIDList(pub SequenceOf<EmergencyAreaID>);
    aper_sequence_of!(EmergencyAreaIDList, EmergencyAreaID, 1, 65535);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct EmergencyAreaIDListForRestart(pub SequenceOf<EmergencyAreaID>);
    aper_sequence_of!(EmergencyAreaIDListForRestart, EmergencyAreaID, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousEmergencyFallbackIndicatorIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousEmergencyFallbackIndicatorIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousEmergencyFallbackIndicatorIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousEmergencyFallbackIndicatorIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousEmergencyFallbackIndicatorIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct EmergencyFallbackIndicatorIEExtensions(pub SequenceOf<AnonymousEmergencyFallbackIndicatorIEExtensions>);
    aper_sequence_of!(
        EmergencyFallbackIndicatorIEExtensions,
        AnonymousEmergencyFallbackIndicatorIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct EmergencyFallbackIndicator {
        #[rasn(identifier = "emergencyFallbackRequestIndicator")]
        pub emergency_fallback_request_indicator: EmergencyFallbackRequestIndicator,
        #[rasn(identifier = "emergencyServiceTargetCN")]
        pub emergency_service_target_cn: Option<EmergencyServiceTargetCN>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<EmergencyFallbackIndicatorIEExtensions>,
    }
    impl EmergencyFallbackIndicator {
        pub fn new(
            emergency_fallback_request_indicator: EmergencyFallbackRequestIndicator,
            emergency_service_target_cn: Option<EmergencyServiceTargetCN>,
            i_e_extensions: Option<EmergencyFallbackIndicatorIEExtensions>,
        ) -> Self {
            Self { emergency_fallback_request_indicator, emergency_service_target_cn, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum EmergencyFallbackRequestIndicator {
        #[rasn(identifier = "emergency-fallback-requested")]
        emergency_fallback_requested = 0,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum EmergencyServiceTargetCN {
        fiveGC = 0,
        epc = 1,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousEndpointIPAddressAndPortIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousEndpointIPAddressAndPortIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousEndpointIPAddressAndPortIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousEndpointIPAddressAndPortIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousEndpointIPAddressAndPortIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct EndpointIPAddressAndPortIEExtensions(pub SequenceOf<AnonymousEndpointIPAddressAndPortIEExtensions>);
    aper_sequence_of!(EndpointIPAddressAndPortIEExtensions, AnonymousEndpointIPAddressAndPortIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct EndpointIPAddressAndPort {
        #[rasn(identifier = "endpointIPAddress")]
        pub endpoint_ipaddress: TransportLayerAddress,
        #[rasn(identifier = "portNumber")]
        pub port_number: PortNumber,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<EndpointIPAddressAndPortIEExtensions>,
    }
    impl EndpointIPAddressAndPort {
        pub fn new(
            endpoint_ipaddress: TransportLayerAddress,
            port_number: PortNumber,
            i_e_extensions: Option<EndpointIPAddressAndPortIEExtensions>,
        ) -> Self {
            Self { endpoint_ipaddress, port_number, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=15"))]
    pub struct EquivalentPLMNs(pub SequenceOf<PLMNIdentity>);
    aper_sequence_of!(EquivalentPLMNs, PLMNIdentity, 1, 15);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum EventType {
        direct = 0,
        #[rasn(identifier = "change-of-serve-cell")]
        change_of_serve_cell = 1,
        #[rasn(identifier = "ue-presence-in-area-of-interest")]
        ue_presence_in_area_of_interest = 2,
        #[rasn(identifier = "stop-change-of-serve-cell")]
        stop_change_of_serve_cell = 3,
        #[rasn(identifier = "stop-ue-presence-in-area-of-interest")]
        stop_ue_presence_in_area_of_interest = 4,
        #[rasn(identifier = "cancel-location-reporting-for-the-ue")]
        cancel_location_reporting_for_the_ue = 5,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("1..=181", extensible))]
    pub struct ExpectedActivityPeriod(pub Integer);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum ExpectedHOInterval {
        sec15 = 0,
        sec30 = 1,
        sec60 = 2,
        sec90 = 3,
        sec120 = 4,
        sec180 = 5,
        #[rasn(identifier = "long-time")]
        long_time = 6,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("1..=181", extensible))]
    pub struct ExpectedIdlePeriod(pub Integer);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousExpectedUEActivityBehaviourIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousExpectedUEActivityBehaviourIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousExpectedUEActivityBehaviourIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousExpectedUEActivityBehaviourIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousExpectedUEActivityBehaviourIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct ExpectedUEActivityBehaviourIEExtensions(
        pub SequenceOf<AnonymousExpectedUEActivityBehaviourIEExtensions>,
    );
    aper_sequence_of!(
        ExpectedUEActivityBehaviourIEExtensions,
        AnonymousExpectedUEActivityBehaviourIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct ExpectedUEActivityBehaviour {
        #[rasn(identifier = "expectedActivityPeriod")]
        pub expected_activity_period: Option<ExpectedActivityPeriod>,
        #[rasn(identifier = "expectedIdlePeriod")]
        pub expected_idle_period: Option<ExpectedIdlePeriod>,
        #[rasn(identifier = "sourceOfUEActivityBehaviourInformation")]
        pub source_of_ueactivity_behaviour_information: Option<SourceOfUEActivityBehaviourInformation>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<ExpectedUEActivityBehaviourIEExtensions>,
    }
    impl ExpectedUEActivityBehaviour {
        pub fn new(
            expected_activity_period: Option<ExpectedActivityPeriod>,
            expected_idle_period: Option<ExpectedIdlePeriod>,
            source_of_ueactivity_behaviour_information: Option<SourceOfUEActivityBehaviourInformation>,
            i_e_extensions: Option<ExpectedUEActivityBehaviourIEExtensions>,
        ) -> Self {
            Self {
                expected_activity_period,
                expected_idle_period,
                source_of_ueactivity_behaviour_information,
                i_e_extensions,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousExpectedUEBehaviourIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousExpectedUEBehaviourIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousExpectedUEBehaviourIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousExpectedUEBehaviourIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousExpectedUEBehaviourIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct ExpectedUEBehaviourIEExtensions(pub SequenceOf<AnonymousExpectedUEBehaviourIEExtensions>);
    aper_sequence_of!(ExpectedUEBehaviourIEExtensions, AnonymousExpectedUEBehaviourIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct ExpectedUEBehaviour {
        #[rasn(identifier = "expectedUEActivityBehaviour")]
        pub expected_ueactivity_behaviour: Option<ExpectedUEActivityBehaviour>,
        #[rasn(identifier = "expectedHOInterval")]
        pub expected_hointerval: Option<ExpectedHOInterval>,
        #[rasn(identifier = "expectedUEMobility")]
        pub expected_uemobility: Option<ExpectedUEMobility>,
        #[rasn(identifier = "expectedUEMovingTrajectory")]
        pub expected_uemoving_trajectory: Option<ExpectedUEMovingTrajectory>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<ExpectedUEBehaviourIEExtensions>,
    }
    impl ExpectedUEBehaviour {
        pub fn new(
            expected_ueactivity_behaviour: Option<ExpectedUEActivityBehaviour>,
            expected_hointerval: Option<ExpectedHOInterval>,
            expected_uemobility: Option<ExpectedUEMobility>,
            expected_uemoving_trajectory: Option<ExpectedUEMovingTrajectory>,
            i_e_extensions: Option<ExpectedUEBehaviourIEExtensions>,
        ) -> Self {
            Self {
                expected_ueactivity_behaviour,
                expected_hointerval,
                expected_uemobility,
                expected_uemoving_trajectory,
                i_e_extensions,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum ExpectedUEMobility {
        stationary = 0,
        mobile = 1,
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"))]
    pub struct ExpectedUEMovingTrajectory(pub SequenceOf<ExpectedUEMovingTrajectoryItem>);
    aper_sequence_of!(ExpectedUEMovingTrajectory, ExpectedUEMovingTrajectoryItem, 1, 16);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousExpectedUEMovingTrajectoryItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousExpectedUEMovingTrajectoryItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousExpectedUEMovingTrajectoryItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousExpectedUEMovingTrajectoryItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousExpectedUEMovingTrajectoryItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct ExpectedUEMovingTrajectoryItemIEExtensions(
        pub SequenceOf<AnonymousExpectedUEMovingTrajectoryItemIEExtensions>,
    );
    aper_sequence_of!(
        ExpectedUEMovingTrajectoryItemIEExtensions,
        AnonymousExpectedUEMovingTrajectoryItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct ExpectedUEMovingTrajectoryItem {
        #[rasn(identifier = "nGRAN-CGI")]
        pub n_gran_cgi: NGRANCGI,
        #[rasn(value("0..=4095"), identifier = "timeStayedInCell")]
        pub time_stayed_in_cell: Option<u16>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<ExpectedUEMovingTrajectoryItemIEExtensions>,
    }
    impl ExpectedUEMovingTrajectoryItem {
        pub fn new(
            n_gran_cgi: NGRANCGI,
            time_stayed_in_cell: Option<u16>,
            i_e_extensions: Option<ExpectedUEMovingTrajectoryItemIEExtensions>,
        ) -> Self {
            Self { n_gran_cgi, time_stayed_in_cell, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousFiveGSTMSIIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousFiveGSTMSIIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousFiveGSTMSIIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousFiveGSTMSIIEExtensions {
        pub fn new(id: u16, criticality: AnonymousFiveGSTMSIIEExtensionsCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct FiveGSTMSIIEExtensions(pub SequenceOf<AnonymousFiveGSTMSIIEExtensions>);
    aper_sequence_of!(FiveGSTMSIIEExtensions, AnonymousFiveGSTMSIIEExtensions, 1, 65535);
    #[doc = " F"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "FiveG-S-TMSI")]
    #[non_exhaustive]
    pub struct FiveGSTMSI {
        #[rasn(identifier = "aMFSetID")]
        pub a_mfset_id: AMFSetID,
        #[rasn(identifier = "aMFPointer")]
        pub a_mfpointer: AMFPointer,
        #[rasn(identifier = "fiveG-TMSI")]
        pub five_g_tmsi: FiveGTMSI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<FiveGSTMSIIEExtensions>,
    }
    impl FiveGSTMSI {
        pub fn new(
            a_mfset_id: AMFSetID,
            a_mfpointer: AMFPointer,
            five_g_tmsi: FiveGTMSI,
            i_e_extensions: Option<FiveGSTMSIIEExtensions>,
        ) -> Self {
            Self { a_mfset_id, a_mfpointer, five_g_tmsi, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "FiveG-TMSI")]
    pub struct FiveGTMSI(pub FixedOctetString<4usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=255", extensible))]
    pub struct FiveQI(pub Integer);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"))]
    pub struct ForbiddenAreaInformation(pub SequenceOf<ForbiddenAreaInformationItem>);
    aper_sequence_of!(ForbiddenAreaInformation, ForbiddenAreaInformationItem, 1, 16);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousForbiddenAreaInformationItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousForbiddenAreaInformationItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousForbiddenAreaInformationItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousForbiddenAreaInformationItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousForbiddenAreaInformationItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct ForbiddenAreaInformationItemIEExtensions(
        pub SequenceOf<AnonymousForbiddenAreaInformationItemIEExtensions>,
    );
    aper_sequence_of!(
        ForbiddenAreaInformationItemIEExtensions,
        AnonymousForbiddenAreaInformationItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "ForbiddenAreaInformation-Item")]
    #[non_exhaustive]
    pub struct ForbiddenAreaInformationItem {
        #[rasn(identifier = "pLMNIdentity")]
        pub p_lmnidentity: PLMNIdentity,
        #[rasn(identifier = "forbiddenTACs")]
        pub forbidden_tacs: ForbiddenTACs,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<ForbiddenAreaInformationItemIEExtensions>,
    }
    impl ForbiddenAreaInformationItem {
        pub fn new(
            p_lmnidentity: PLMNIdentity,
            forbidden_tacs: ForbiddenTACs,
            i_e_extensions: Option<ForbiddenAreaInformationItemIEExtensions>,
        ) -> Self {
            Self { p_lmnidentity, forbidden_tacs, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=4096"))]
    pub struct ForbiddenTACs(pub SequenceOf<TAC>);
    aper_sequence_of!(ForbiddenTACs, TAC, 1, 4096);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousGBRQosInformationIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousGBRQosInformationIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousGBRQosInformationIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousGBRQosInformationIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousGBRQosInformationIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct GBRQosInformationIEExtensions(pub SequenceOf<AnonymousGBRQosInformationIEExtensions>);
    aper_sequence_of!(GBRQosInformationIEExtensions, AnonymousGBRQosInformationIEExtensions, 1, 65535);
    #[doc = " G"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "GBR-QosInformation")]
    #[non_exhaustive]
    pub struct GBRQosInformation {
        #[rasn(identifier = "maximumFlowBitRateDL")]
        pub maximum_flow_bit_rate_dl: BitRate,
        #[rasn(identifier = "maximumFlowBitRateUL")]
        pub maximum_flow_bit_rate_ul: BitRate,
        #[rasn(identifier = "guaranteedFlowBitRateDL")]
        pub guaranteed_flow_bit_rate_dl: BitRate,
        #[rasn(identifier = "guaranteedFlowBitRateUL")]
        pub guaranteed_flow_bit_rate_ul: BitRate,
        #[rasn(identifier = "notificationControl")]
        pub notification_control: Option<NotificationControl>,
        #[rasn(identifier = "maximumPacketLossRateDL")]
        pub maximum_packet_loss_rate_dl: Option<PacketLossRate>,
        #[rasn(identifier = "maximumPacketLossRateUL")]
        pub maximum_packet_loss_rate_ul: Option<PacketLossRate>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<GBRQosInformationIEExtensions>,
    }
    impl GBRQosInformation {
        pub fn new(
            maximum_flow_bit_rate_dl: BitRate,
            maximum_flow_bit_rate_ul: BitRate,
            guaranteed_flow_bit_rate_dl: BitRate,
            guaranteed_flow_bit_rate_ul: BitRate,
            notification_control: Option<NotificationControl>,
            maximum_packet_loss_rate_dl: Option<PacketLossRate>,
            maximum_packet_loss_rate_ul: Option<PacketLossRate>,
            i_e_extensions: Option<GBRQosInformationIEExtensions>,
        ) -> Self {
            Self {
                maximum_flow_bit_rate_dl,
                maximum_flow_bit_rate_ul,
                guaranteed_flow_bit_rate_dl,
                guaranteed_flow_bit_rate_ul,
                notification_control,
                maximum_packet_loss_rate_dl,
                maximum_packet_loss_rate_ul,
                i_e_extensions,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum GNBIDChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct GNBIDChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: GNBIDChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl GNBIDChoiceExtensions {
        pub fn new(id: u16, criticality: GNBIDChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags, identifier = "GNB-ID")]
    pub enum GNBID {
        #[rasn(size("22..=32"), identifier = "gNB-ID")]
        gNB_ID(BitString),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(GNBIDChoiceExtensions),
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "GTP-TEID")]
    pub struct GTPTEID(pub FixedOctetString<4usize>);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousGTPTunnelIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousGTPTunnelIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousGTPTunnelIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousGTPTunnelIEExtensions {
        pub fn new(id: u16, criticality: AnonymousGTPTunnelIEExtensionsCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct GTPTunnelIEExtensions(pub SequenceOf<AnonymousGTPTunnelIEExtensions>);
    aper_sequence_of!(GTPTunnelIEExtensions, AnonymousGTPTunnelIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct GTPTunnel {
        #[rasn(identifier = "transportLayerAddress")]
        pub transport_layer_address: TransportLayerAddress,
        #[rasn(identifier = "gTP-TEID")]
        pub g_tp_teid: GTPTEID,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<GTPTunnelIEExtensions>,
    }
    impl GTPTunnel {
        pub fn new(
            transport_layer_address: TransportLayerAddress,
            g_tp_teid: GTPTEID,
            i_e_extensions: Option<GTPTunnelIEExtensions>,
        ) -> Self {
            Self { transport_layer_address, g_tp_teid, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousGUAMIIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousGUAMIIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousGUAMIIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousGUAMIIEExtensions {
        pub fn new(id: u16, criticality: AnonymousGUAMIIEExtensionsCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct GUAMIIEExtensions(pub SequenceOf<AnonymousGUAMIIEExtensions>);
    aper_sequence_of!(GUAMIIEExtensions, AnonymousGUAMIIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct GUAMI {
        #[rasn(identifier = "pLMNIdentity")]
        pub p_lmnidentity: PLMNIdentity,
        #[rasn(identifier = "aMFRegionID")]
        pub a_mfregion_id: AMFRegionID,
        #[rasn(identifier = "aMFSetID")]
        pub a_mfset_id: AMFSetID,
        #[rasn(identifier = "aMFPointer")]
        pub a_mfpointer: AMFPointer,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<GUAMIIEExtensions>,
    }
    impl GUAMI {
        pub fn new(
            p_lmnidentity: PLMNIdentity,
            a_mfregion_id: AMFRegionID,
            a_mfset_id: AMFSetID,
            a_mfpointer: AMFPointer,
            i_e_extensions: Option<GUAMIIEExtensions>,
        ) -> Self {
            Self { p_lmnidentity, a_mfregion_id, a_mfset_id, a_mfpointer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousGlobalGNBIDIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousGlobalGNBIDIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousGlobalGNBIDIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousGlobalGNBIDIEExtensions {
        pub fn new(id: u16, criticality: AnonymousGlobalGNBIDIEExtensionsCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct GlobalGNBIDIEExtensions(pub SequenceOf<AnonymousGlobalGNBIDIEExtensions>);
    aper_sequence_of!(GlobalGNBIDIEExtensions, AnonymousGlobalGNBIDIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "GlobalGNB-ID")]
    #[non_exhaustive]
    pub struct GlobalGNBID {
        #[rasn(identifier = "pLMNIdentity")]
        pub p_lmnidentity: PLMNIdentity,
        #[rasn(identifier = "gNB-ID")]
        pub g_nb_id: GNBID,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<GlobalGNBIDIEExtensions>,
    }
    impl GlobalGNBID {
        pub fn new(
            p_lmnidentity: PLMNIdentity,
            g_nb_id: GNBID,
            i_e_extensions: Option<GlobalGNBIDIEExtensions>,
        ) -> Self {
            Self { p_lmnidentity, g_nb_id, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousGlobalN3IWFIDIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousGlobalN3IWFIDIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousGlobalN3IWFIDIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousGlobalN3IWFIDIEExtensions {
        pub fn new(id: u16, criticality: AnonymousGlobalN3IWFIDIEExtensionsCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct GlobalN3IWFIDIEExtensions(pub SequenceOf<AnonymousGlobalN3IWFIDIEExtensions>);
    aper_sequence_of!(GlobalN3IWFIDIEExtensions, AnonymousGlobalN3IWFIDIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "GlobalN3IWF-ID")]
    #[non_exhaustive]
    pub struct GlobalN3IWFID {
        #[rasn(identifier = "pLMNIdentity")]
        pub p_lmnidentity: PLMNIdentity,
        #[rasn(identifier = "n3IWF-ID")]
        pub n3_iwf_id: N3IWFID,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<GlobalN3IWFIDIEExtensions>,
    }
    impl GlobalN3IWFID {
        pub fn new(
            p_lmnidentity: PLMNIdentity,
            n3_iwf_id: N3IWFID,
            i_e_extensions: Option<GlobalN3IWFIDIEExtensions>,
        ) -> Self {
            Self { p_lmnidentity, n3_iwf_id, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousGlobalNgENBIDIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousGlobalNgENBIDIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousGlobalNgENBIDIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousGlobalNgENBIDIEExtensions {
        pub fn new(id: u16, criticality: AnonymousGlobalNgENBIDIEExtensionsCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct GlobalNgENBIDIEExtensions(pub SequenceOf<AnonymousGlobalNgENBIDIEExtensions>);
    aper_sequence_of!(GlobalNgENBIDIEExtensions, AnonymousGlobalNgENBIDIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "GlobalNgENB-ID")]
    #[non_exhaustive]
    pub struct GlobalNgENBID {
        #[rasn(identifier = "pLMNIdentity")]
        pub p_lmnidentity: PLMNIdentity,
        #[rasn(identifier = "ngENB-ID")]
        pub ng_enb_id: NgENBID,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<GlobalNgENBIDIEExtensions>,
    }
    impl GlobalNgENBID {
        pub fn new(
            p_lmnidentity: PLMNIdentity,
            ng_enb_id: NgENBID,
            i_e_extensions: Option<GlobalNgENBIDIEExtensions>,
        ) -> Self {
            Self { p_lmnidentity, ng_enb_id, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum GlobalRANNodeIDChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct GlobalRANNodeIDChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: GlobalRANNodeIDChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl GlobalRANNodeIDChoiceExtensions {
        pub fn new(id: u16, criticality: GlobalRANNodeIDChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum GlobalRANNodeID {
        #[rasn(identifier = "globalGNB-ID")]
        globalGNB_ID(GlobalGNBID),
        #[rasn(identifier = "globalNgENB-ID")]
        globalNgENB_ID(GlobalNgENBID),
        #[rasn(identifier = "globalN3IWF-ID")]
        globalN3IWF_ID(GlobalN3IWFID),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(GlobalRANNodeIDChoiceExtensions),
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverCommandTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverCommandTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverCommandTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousHandoverCommandTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousHandoverCommandTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct HandoverCommandTransferIEExtensions(pub SequenceOf<AnonymousHandoverCommandTransferIEExtensions>);
    aper_sequence_of!(HandoverCommandTransferIEExtensions, AnonymousHandoverCommandTransferIEExtensions, 1, 65535);
    #[doc = " H"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverCommandTransfer {
        #[rasn(identifier = "dLForwardingUP-TNLInformation")]
        pub d_lforwarding_up_tnlinformation: Option<UPTransportLayerInformation>,
        #[rasn(identifier = "qosFlowToBeForwardedList")]
        pub qos_flow_to_be_forwarded_list: Option<QosFlowToBeForwardedList>,
        #[rasn(identifier = "dataForwardingResponseDRBList")]
        pub data_forwarding_response_drblist: Option<DataForwardingResponseDRBList>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<HandoverCommandTransferIEExtensions>,
    }
    impl HandoverCommandTransfer {
        pub fn new(
            d_lforwarding_up_tnlinformation: Option<UPTransportLayerInformation>,
            qos_flow_to_be_forwarded_list: Option<QosFlowToBeForwardedList>,
            data_forwarding_response_drblist: Option<DataForwardingResponseDRBList>,
            i_e_extensions: Option<HandoverCommandTransferIEExtensions>,
        ) -> Self {
            Self {
                d_lforwarding_up_tnlinformation,
                qos_flow_to_be_forwarded_list,
                data_forwarding_response_drblist,
                i_e_extensions,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum HandoverFlag {
        #[rasn(identifier = "handover-preparation")]
        handover_preparation = 0,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverPreparationUnsuccessfulTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverPreparationUnsuccessfulTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverPreparationUnsuccessfulTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousHandoverPreparationUnsuccessfulTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousHandoverPreparationUnsuccessfulTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct HandoverPreparationUnsuccessfulTransferIEExtensions(
        pub SequenceOf<AnonymousHandoverPreparationUnsuccessfulTransferIEExtensions>,
    );
    aper_sequence_of!(
        HandoverPreparationUnsuccessfulTransferIEExtensions,
        AnonymousHandoverPreparationUnsuccessfulTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverPreparationUnsuccessfulTransfer {
        pub cause: Cause,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<HandoverPreparationUnsuccessfulTransferIEExtensions>,
    }
    impl HandoverPreparationUnsuccessfulTransfer {
        pub fn new(cause: Cause, i_e_extensions: Option<HandoverPreparationUnsuccessfulTransferIEExtensions>) -> Self {
            Self { cause, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverRequestAcknowledgeTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverRequestAcknowledgeTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverRequestAcknowledgeTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousHandoverRequestAcknowledgeTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousHandoverRequestAcknowledgeTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct HandoverRequestAcknowledgeTransferIEExtensions(
        pub SequenceOf<AnonymousHandoverRequestAcknowledgeTransferIEExtensions>,
    );
    aper_sequence_of!(
        HandoverRequestAcknowledgeTransferIEExtensions,
        AnonymousHandoverRequestAcknowledgeTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverRequestAcknowledgeTransfer {
        #[rasn(identifier = "dL-NGU-UP-TNLInformation")]
        pub d_l_ngu_up_tnlinformation: UPTransportLayerInformation,
        #[rasn(identifier = "dLForwardingUP-TNLInformation")]
        pub d_lforwarding_up_tnlinformation: Option<UPTransportLayerInformation>,
        #[rasn(identifier = "securityResult")]
        pub security_result: Option<SecurityResult>,
        #[rasn(identifier = "qosFlowSetupResponseList")]
        pub qos_flow_setup_response_list: QosFlowListWithDataForwarding,
        #[rasn(identifier = "qosFlowFailedToSetupList")]
        pub qos_flow_failed_to_setup_list: Option<QosFlowListWithCause>,
        #[rasn(identifier = "dataForwardingResponseDRBList")]
        pub data_forwarding_response_drblist: Option<DataForwardingResponseDRBList>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<HandoverRequestAcknowledgeTransferIEExtensions>,
    }
    impl HandoverRequestAcknowledgeTransfer {
        pub fn new(
            d_l_ngu_up_tnlinformation: UPTransportLayerInformation,
            d_lforwarding_up_tnlinformation: Option<UPTransportLayerInformation>,
            security_result: Option<SecurityResult>,
            qos_flow_setup_response_list: QosFlowListWithDataForwarding,
            qos_flow_failed_to_setup_list: Option<QosFlowListWithCause>,
            data_forwarding_response_drblist: Option<DataForwardingResponseDRBList>,
            i_e_extensions: Option<HandoverRequestAcknowledgeTransferIEExtensions>,
        ) -> Self {
            Self {
                d_l_ngu_up_tnlinformation,
                d_lforwarding_up_tnlinformation,
                security_result,
                qos_flow_setup_response_list,
                qos_flow_failed_to_setup_list,
                data_forwarding_response_drblist,
                i_e_extensions,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverRequiredTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverRequiredTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverRequiredTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousHandoverRequiredTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousHandoverRequiredTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct HandoverRequiredTransferIEExtensions(pub SequenceOf<AnonymousHandoverRequiredTransferIEExtensions>);
    aper_sequence_of!(HandoverRequiredTransferIEExtensions, AnonymousHandoverRequiredTransferIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverRequiredTransfer {
        #[rasn(identifier = "directForwardingPathAvailability")]
        pub direct_forwarding_path_availability: Option<DirectForwardingPathAvailability>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<HandoverRequiredTransferIEExtensions>,
    }
    impl HandoverRequiredTransfer {
        pub fn new(
            direct_forwarding_path_availability: Option<DirectForwardingPathAvailability>,
            i_e_extensions: Option<HandoverRequiredTransferIEExtensions>,
        ) -> Self {
            Self { direct_forwarding_path_availability, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverResourceAllocationUnsuccessfulTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverResourceAllocationUnsuccessfulTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverResourceAllocationUnsuccessfulTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousHandoverResourceAllocationUnsuccessfulTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousHandoverResourceAllocationUnsuccessfulTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct HandoverResourceAllocationUnsuccessfulTransferIEExtensions(
        pub SequenceOf<AnonymousHandoverResourceAllocationUnsuccessfulTransferIEExtensions>,
    );
    aper_sequence_of!(
        HandoverResourceAllocationUnsuccessfulTransferIEExtensions,
        AnonymousHandoverResourceAllocationUnsuccessfulTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverResourceAllocationUnsuccessfulTransfer {
        pub cause: Cause,
        #[rasn(identifier = "criticalityDiagnostics")]
        pub criticality_diagnostics: Option<CriticalityDiagnostics>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<HandoverResourceAllocationUnsuccessfulTransferIEExtensions>,
    }
    impl HandoverResourceAllocationUnsuccessfulTransfer {
        pub fn new(
            cause: Cause,
            criticality_diagnostics: Option<CriticalityDiagnostics>,
            i_e_extensions: Option<HandoverResourceAllocationUnsuccessfulTransferIEExtensions>,
        ) -> Self {
            Self { cause, criticality_diagnostics, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum HandoverType {
        intra5gs = 0,
        #[rasn(identifier = "fivegs-to-eps")]
        fivegs_to_eps = 1,
        #[rasn(identifier = "eps-to-5gs")]
        eps_to_5gs = 2,
    }
    #[doc = " I"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum IMSVoiceSupportIndicator {
        supported = 0,
        #[rasn(identifier = "not-supported")]
        not_supported = 1,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("1..=256", extensible))]
    pub struct IndexToRFSP(pub Integer);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousInfoOnRecommendedCellsAndRANNodesForPagingIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousInfoOnRecommendedCellsAndRANNodesForPagingIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousInfoOnRecommendedCellsAndRANNodesForPagingIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousInfoOnRecommendedCellsAndRANNodesForPagingIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousInfoOnRecommendedCellsAndRANNodesForPagingIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct InfoOnRecommendedCellsAndRANNodesForPagingIEExtensions(
        pub SequenceOf<AnonymousInfoOnRecommendedCellsAndRANNodesForPagingIEExtensions>,
    );
    aper_sequence_of!(
        InfoOnRecommendedCellsAndRANNodesForPagingIEExtensions,
        AnonymousInfoOnRecommendedCellsAndRANNodesForPagingIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct InfoOnRecommendedCellsAndRANNodesForPaging {
        #[rasn(identifier = "recommendedCellsForPaging")]
        pub recommended_cells_for_paging: RecommendedCellsForPaging,
        #[rasn(identifier = "recommendRANNodesForPaging")]
        pub recommend_rannodes_for_paging: RecommendedRANNodesForPaging,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<InfoOnRecommendedCellsAndRANNodesForPagingIEExtensions>,
    }
    impl InfoOnRecommendedCellsAndRANNodesForPaging {
        pub fn new(
            recommended_cells_for_paging: RecommendedCellsForPaging,
            recommend_rannodes_for_paging: RecommendedRANNodesForPaging,
            i_e_extensions: Option<InfoOnRecommendedCellsAndRANNodesForPagingIEExtensions>,
        ) -> Self {
            Self { recommended_cells_for_paging, recommend_rannodes_for_paging, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum IntegrityProtectionIndication {
        required = 0,
        preferred = 1,
        #[rasn(identifier = "not-needed")]
        not_needed = 2,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum IntegrityProtectionResult {
        performed = 0,
        #[rasn(identifier = "not-performed")]
        not_performed = 1,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("1..=16", extensible))]
    pub struct IntendedNumberOfPagingAttempts(pub Integer);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct InterfacesToTrace(pub FixedBitString<8usize>);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum LastVisitedCellInformationChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct LastVisitedCellInformationChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: LastVisitedCellInformationChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl LastVisitedCellInformationChoiceExtensions {
        pub fn new(id: u16, criticality: LastVisitedCellInformationChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " J"]
    #[doc = " K"]
    #[doc = " L"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum LastVisitedCellInformation {
        nGRANCell(LastVisitedNGRANCellInformation),
        eUTRANCell(LastVisitedEUTRANCellInformation),
        uTRANCell(LastVisitedUTRANCellInformation),
        gERANCell(LastVisitedGERANCellInformation),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(LastVisitedCellInformationChoiceExtensions),
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousLastVisitedCellItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousLastVisitedCellItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousLastVisitedCellItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousLastVisitedCellItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousLastVisitedCellItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct LastVisitedCellItemIEExtensions(pub SequenceOf<AnonymousLastVisitedCellItemIEExtensions>);
    aper_sequence_of!(LastVisitedCellItemIEExtensions, AnonymousLastVisitedCellItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct LastVisitedCellItem {
        #[rasn(identifier = "lastVisitedCellInformation")]
        pub last_visited_cell_information: LastVisitedCellInformation,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<LastVisitedCellItemIEExtensions>,
    }
    impl LastVisitedCellItem {
        pub fn new(
            last_visited_cell_information: LastVisitedCellInformation,
            i_e_extensions: Option<LastVisitedCellItemIEExtensions>,
        ) -> Self {
            Self { last_visited_cell_information, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct LastVisitedEUTRANCellInformation(pub OctetString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct LastVisitedGERANCellInformation(pub OctetString);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousLastVisitedNGRANCellInformationIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousLastVisitedNGRANCellInformationIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousLastVisitedNGRANCellInformationIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousLastVisitedNGRANCellInformationIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousLastVisitedNGRANCellInformationIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct LastVisitedNGRANCellInformationIEExtensions(
        pub SequenceOf<AnonymousLastVisitedNGRANCellInformationIEExtensions>,
    );
    aper_sequence_of!(
        LastVisitedNGRANCellInformationIEExtensions,
        AnonymousLastVisitedNGRANCellInformationIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct LastVisitedNGRANCellInformation {
        #[rasn(identifier = "globalCellID")]
        pub global_cell_id: NGRANCGI,
        #[rasn(identifier = "cellType")]
        pub cell_type: CellType,
        #[rasn(identifier = "timeUEStayedInCell")]
        pub time_uestayed_in_cell: TimeUEStayedInCell,
        #[rasn(identifier = "timeUEStayedInCellEnhancedGranularity")]
        pub time_uestayed_in_cell_enhanced_granularity: Option<TimeUEStayedInCellEnhancedGranularity>,
        #[rasn(identifier = "hOCauseValue")]
        pub h_ocause_value: Option<Cause>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<LastVisitedNGRANCellInformationIEExtensions>,
    }
    impl LastVisitedNGRANCellInformation {
        pub fn new(
            global_cell_id: NGRANCGI,
            cell_type: CellType,
            time_uestayed_in_cell: TimeUEStayedInCell,
            time_uestayed_in_cell_enhanced_granularity: Option<TimeUEStayedInCellEnhancedGranularity>,
            h_ocause_value: Option<Cause>,
            i_e_extensions: Option<LastVisitedNGRANCellInformationIEExtensions>,
        ) -> Self {
            Self {
                global_cell_id,
                cell_type,
                time_uestayed_in_cell,
                time_uestayed_in_cell_enhanced_granularity,
                h_ocause_value,
                i_e_extensions,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct LastVisitedUTRANCellInformation(pub OctetString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum LocationReportingAdditionalInfo {
        includePSCell = 0,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("1..=64", extensible))]
    pub struct LocationReportingReferenceID(pub Integer);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousLocationReportingRequestTypeIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousLocationReportingRequestTypeIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousLocationReportingRequestTypeIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousLocationReportingRequestTypeIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousLocationReportingRequestTypeIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct LocationReportingRequestTypeIEExtensions(
        pub SequenceOf<AnonymousLocationReportingRequestTypeIEExtensions>,
    );
    aper_sequence_of!(
        LocationReportingRequestTypeIEExtensions,
        AnonymousLocationReportingRequestTypeIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct LocationReportingRequestType {
        #[rasn(identifier = "eventType")]
        pub event_type: EventType,
        #[rasn(identifier = "reportArea")]
        pub report_area: ReportArea,
        #[rasn(identifier = "areaOfInterestList")]
        pub area_of_interest_list: Option<AreaOfInterestList>,
        #[rasn(identifier = "locationReportingReferenceIDToBeCancelled")]
        pub location_reporting_reference_idto_be_cancelled: Option<LocationReportingReferenceID>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<LocationReportingRequestTypeIEExtensions>,
    }
    impl LocationReportingRequestType {
        pub fn new(
            event_type: EventType,
            report_area: ReportArea,
            area_of_interest_list: Option<AreaOfInterestList>,
            location_reporting_reference_idto_be_cancelled: Option<LocationReportingReferenceID>,
            i_e_extensions: Option<LocationReportingRequestTypeIEExtensions>,
        ) -> Self {
            Self {
                event_type,
                report_area,
                area_of_interest_list,
                location_reporting_reference_idto_be_cancelled,
                i_e_extensions,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum MICOModeIndication {
        #[rasn(identifier = "true")]
        R_true = 0,
    }
    #[doc = " M"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct MaskedIMEISV(pub FixedBitString<64usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=4095", extensible))]
    pub struct MaximumDataBurstVolume(pub Integer);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum MaximumIntegrityProtectedDataRate {
        bitrate64kbs = 0,
        #[rasn(identifier = "maximum-UE-rate")]
        maximum_UE_rate = 1,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct MessageIdentifier(pub FixedBitString<16usize>);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousMobilityRestrictionListIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousMobilityRestrictionListIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousMobilityRestrictionListIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousMobilityRestrictionListIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousMobilityRestrictionListIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct MobilityRestrictionListIEExtensions(pub SequenceOf<AnonymousMobilityRestrictionListIEExtensions>);
    aper_sequence_of!(MobilityRestrictionListIEExtensions, AnonymousMobilityRestrictionListIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct MobilityRestrictionList {
        #[rasn(identifier = "servingPLMN")]
        pub serving_plmn: PLMNIdentity,
        #[rasn(identifier = "equivalentPLMNs")]
        pub equivalent_plmns: Option<EquivalentPLMNs>,
        #[rasn(identifier = "rATRestrictions")]
        pub r_atrestrictions: Option<RATRestrictions>,
        #[rasn(identifier = "forbiddenAreaInformation")]
        pub forbidden_area_information: Option<ForbiddenAreaInformation>,
        #[rasn(identifier = "serviceAreaInformation")]
        pub service_area_information: Option<ServiceAreaInformation>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<MobilityRestrictionListIEExtensions>,
    }
    impl MobilityRestrictionList {
        pub fn new(
            serving_plmn: PLMNIdentity,
            equivalent_plmns: Option<EquivalentPLMNs>,
            r_atrestrictions: Option<RATRestrictions>,
            forbidden_area_information: Option<ForbiddenAreaInformation>,
            service_area_information: Option<ServiceAreaInformation>,
            i_e_extensions: Option<MobilityRestrictionListIEExtensions>,
        ) -> Self {
            Self {
                serving_plmn,
                equivalent_plmns,
                r_atrestrictions,
                forbidden_area_information,
                service_area_information,
                i_e_extensions,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum N3IWFIDChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct N3IWFIDChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: N3IWFIDChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl N3IWFIDChoiceExtensions {
        pub fn new(id: u16, criticality: N3IWFIDChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " N"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags, identifier = "N3IWF-ID")]
    pub enum N3IWFID {
        #[rasn(size("16"), identifier = "n3IWF-ID")]
        n3IWF_ID(BitString),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(N3IWFIDChoiceExtensions),
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "NAS-PDU")]
    pub struct NASPDU(pub OctetString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct NASSecurityParametersFromNGRAN(pub OctetString);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum NGRANCGIChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct NGRANCGIChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: NGRANCGIChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl NGRANCGIChoiceExtensions {
        pub fn new(id: u16, criticality: NGRANCGIChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags, identifier = "NGRAN-CGI")]
    pub enum NGRANCGI {
        #[rasn(identifier = "nR-CGI")]
        nR_CGI(NRCGI),
        #[rasn(identifier = "eUTRA-CGI")]
        eUTRA_CGI(EUTRACGI),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(NGRANCGIChoiceExtensions),
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousNGRANTNLAssociationToRemoveItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousNGRANTNLAssociationToRemoveItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousNGRANTNLAssociationToRemoveItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousNGRANTNLAssociationToRemoveItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousNGRANTNLAssociationToRemoveItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct NGRANTNLAssociationToRemoveItemIEExtensions(
        pub SequenceOf<AnonymousNGRANTNLAssociationToRemoveItemIEExtensions>,
    );
    aper_sequence_of!(
        NGRANTNLAssociationToRemoveItemIEExtensions,
        AnonymousNGRANTNLAssociationToRemoveItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "NGRAN-TNLAssociationToRemoveItem")]
    pub struct NGRANTNLAssociationToRemoveItem {
        #[rasn(identifier = "tNLAssociationTransportLayerAddress")]
        pub t_nlassociation_transport_layer_address: CPTransportLayerInformation,
        #[rasn(identifier = "tNLAssociationTransportLayerAddressAMF")]
        pub t_nlassociation_transport_layer_address_amf: Option<CPTransportLayerInformation>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<NGRANTNLAssociationToRemoveItemIEExtensions>,
    }
    impl NGRANTNLAssociationToRemoveItem {
        pub fn new(
            t_nlassociation_transport_layer_address: CPTransportLayerInformation,
            t_nlassociation_transport_layer_address_amf: Option<CPTransportLayerInformation>,
            i_e_extensions: Option<NGRANTNLAssociationToRemoveItemIEExtensions>,
        ) -> Self {
            Self {
                t_nlassociation_transport_layer_address,
                t_nlassociation_transport_layer_address_amf,
                i_e_extensions,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=32"), identifier = "NGRAN-TNLAssociationToRemoveList")]
    pub struct NGRANTNLAssociationToRemoveList(pub SequenceOf<NGRANTNLAssociationToRemoveItem>);
    aper_sequence_of!(NGRANTNLAssociationToRemoveList, NGRANTNLAssociationToRemoveItem, 1, 32);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct NGRANTraceID(pub FixedOctetString<8usize>);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousNRCGIIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousNRCGIIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousNRCGIIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousNRCGIIEExtensions {
        pub fn new(id: u16, criticality: AnonymousNRCGIIEExtensionsCriticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct NRCGIIEExtensions(pub SequenceOf<AnonymousNRCGIIEExtensions>);
    aper_sequence_of!(NRCGIIEExtensions, AnonymousNRCGIIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "NR-CGI")]
    #[non_exhaustive]
    pub struct NRCGI {
        #[rasn(identifier = "pLMNIdentity")]
        pub p_lmnidentity: PLMNIdentity,
        #[rasn(identifier = "nRCellIdentity")]
        pub n_rcell_identity: NRCellIdentity,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<NRCGIIEExtensions>,
    }
    impl NRCGI {
        pub fn new(
            p_lmnidentity: PLMNIdentity,
            n_rcell_identity: NRCellIdentity,
            i_e_extensions: Option<NRCGIIEExtensions>,
        ) -> Self {
            Self { p_lmnidentity, n_rcell_identity, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16384"), identifier = "NR-CGIList")]
    pub struct NRCGIList(pub SequenceOf<NRCGI>);
    aper_sequence_of!(NRCGIList, NRCGI, 1, 16384);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"), identifier = "NR-CGIListForWarning")]
    pub struct NRCGIListForWarning(pub SequenceOf<NRCGI>);
    aper_sequence_of!(NRCGIListForWarning, NRCGI, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct NRCellIdentity(pub FixedBitString<36usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "NRPPa-PDU")]
    pub struct NRPPaPDU(pub OctetString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("16", extensible))]
    pub struct NRencryptionAlgorithms(pub BitString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("16", extensible))]
    pub struct NRintegrityProtectionAlgorithms(pub BitString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("1..=256", extensible))]
    pub struct NetworkInstance(pub Integer);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum NewSecurityContextInd {
        #[rasn(identifier = "true")]
        R_true = 0,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=7"))]
    pub struct NextHopChainingCount(pub u8);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum NextPagingAreaScope {
        same = 0,
        changed = 1,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum NgENBIDChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct NgENBIDChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: NgENBIDChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl NgENBIDChoiceExtensions {
        pub fn new(id: u16, criticality: NgENBIDChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags, identifier = "NgENB-ID")]
    pub enum NgENBID {
        #[rasn(size("20"), identifier = "macroNgENB-ID")]
        macroNgENB_ID(BitString),
        #[rasn(size("18"), identifier = "shortMacroNgENB-ID")]
        shortMacroNgENB_ID(BitString),
        #[rasn(size("21"), identifier = "longMacroNgENB-ID")]
        longMacroNgENB_ID(BitString),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(NgENBIDChoiceExtensions),
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousNonDynamic5QIDescriptorIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousNonDynamic5QIDescriptorIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousNonDynamic5QIDescriptorIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousNonDynamic5QIDescriptorIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousNonDynamic5QIDescriptorIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct NonDynamic5QIDescriptorIEExtensions(pub SequenceOf<AnonymousNonDynamic5QIDescriptorIEExtensions>);
    aper_sequence_of!(NonDynamic5QIDescriptorIEExtensions, AnonymousNonDynamic5QIDescriptorIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct NonDynamic5QIDescriptor {
        #[rasn(identifier = "fiveQI")]
        pub five_qi: FiveQI,
        #[rasn(identifier = "priorityLevelQos")]
        pub priority_level_qos: Option<PriorityLevelQos>,
        #[rasn(identifier = "averagingWindow")]
        pub averaging_window: Option<AveragingWindow>,
        #[rasn(identifier = "maximumDataBurstVolume")]
        pub maximum_data_burst_volume: Option<MaximumDataBurstVolume>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<NonDynamic5QIDescriptorIEExtensions>,
    }
    impl NonDynamic5QIDescriptor {
        pub fn new(
            five_qi: FiveQI,
            priority_level_qos: Option<PriorityLevelQos>,
            averaging_window: Option<AveragingWindow>,
            maximum_data_burst_volume: Option<MaximumDataBurstVolume>,
            i_e_extensions: Option<NonDynamic5QIDescriptorIEExtensions>,
        ) -> Self {
            Self { five_qi, priority_level_qos, averaging_window, maximum_data_burst_volume, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"))]
    pub struct NotAllowedTACs(pub SequenceOf<TAC>);
    aper_sequence_of!(NotAllowedTACs, TAC, 1, 16);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum NotificationCause {
        fulfilled = 0,
        #[rasn(identifier = "not-fulfilled")]
        not_fulfilled = 1,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum NotificationControl {
        #[rasn(identifier = "notification-requested")]
        notification_requested = 0,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=65535"))]
    pub struct NumberOfBroadcasts(pub u16);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=65535"))]
    pub struct NumberOfBroadcastsRequested(pub u16);
    #[doc = " O"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum OverloadAction {
        #[rasn(identifier = "reject-non-emergency-mo-dt")]
        reject_non_emergency_mo_dt = 0,
        #[rasn(identifier = "reject-rrc-cr-signalling")]
        reject_rrc_cr_signalling = 1,
        #[rasn(identifier = "permit-emergency-sessions-and-mobile-terminated-services-only")]
        permit_emergency_sessions_and_mobile_terminated_services_only = 2,
        #[rasn(identifier = "permit-high-priority-sessions-and-mobile-terminated-services-only")]
        permit_high_priority_sessions_and_mobile_terminated_services_only = 3,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum OverloadResponseChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct OverloadResponseChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: OverloadResponseChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl OverloadResponseChoiceExtensions {
        pub fn new(id: u16, criticality: OverloadResponseChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum OverloadResponse {
        overloadAction(OverloadAction),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(OverloadResponseChoiceExtensions),
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousOverloadStartNSSAIItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousOverloadStartNSSAIItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousOverloadStartNSSAIItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousOverloadStartNSSAIItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousOverloadStartNSSAIItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct OverloadStartNSSAIItemIEExtensions(pub SequenceOf<AnonymousOverloadStartNSSAIItemIEExtensions>);
    aper_sequence_of!(OverloadStartNSSAIItemIEExtensions, AnonymousOverloadStartNSSAIItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct OverloadStartNSSAIItem {
        #[rasn(identifier = "sliceOverloadList")]
        pub slice_overload_list: SliceOverloadList,
        #[rasn(identifier = "sliceOverloadResponse")]
        pub slice_overload_response: Option<OverloadResponse>,
        #[rasn(identifier = "sliceTrafficLoadReductionIndication")]
        pub slice_traffic_load_reduction_indication: Option<TrafficLoadReductionIndication>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<OverloadStartNSSAIItemIEExtensions>,
    }
    impl OverloadStartNSSAIItem {
        pub fn new(
            slice_overload_list: SliceOverloadList,
            slice_overload_response: Option<OverloadResponse>,
            slice_traffic_load_reduction_indication: Option<TrafficLoadReductionIndication>,
            i_e_extensions: Option<OverloadStartNSSAIItemIEExtensions>,
        ) -> Self {
            Self {
                slice_overload_list,
                slice_overload_response,
                slice_traffic_load_reduction_indication,
                i_e_extensions,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=1024"))]
    pub struct OverloadStartNSSAIList(pub SequenceOf<OverloadStartNSSAIItem>);
    aper_sequence_of!(OverloadStartNSSAIList, OverloadStartNSSAIItem, 1, 1024);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionAggregateMaximumBitRateIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionAggregateMaximumBitRateIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionAggregateMaximumBitRateIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionAggregateMaximumBitRateIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionAggregateMaximumBitRateIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionAggregateMaximumBitRateIEExtensions(
        pub SequenceOf<AnonymousPDUSessionAggregateMaximumBitRateIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionAggregateMaximumBitRateIEExtensions,
        AnonymousPDUSessionAggregateMaximumBitRateIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionAggregateMaximumBitRate {
        #[rasn(identifier = "pDUSessionAggregateMaximumBitRateDL")]
        pub p_dusession_aggregate_maximum_bit_rate_dl: BitRate,
        #[rasn(identifier = "pDUSessionAggregateMaximumBitRateUL")]
        pub p_dusession_aggregate_maximum_bit_rate_ul: BitRate,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionAggregateMaximumBitRateIEExtensions>,
    }
    impl PDUSessionAggregateMaximumBitRate {
        pub fn new(
            p_dusession_aggregate_maximum_bit_rate_dl: BitRate,
            p_dusession_aggregate_maximum_bit_rate_ul: BitRate,
            i_e_extensions: Option<PDUSessionAggregateMaximumBitRateIEExtensions>,
        ) -> Self {
            Self {
                p_dusession_aggregate_maximum_bit_rate_dl,
                p_dusession_aggregate_maximum_bit_rate_ul,
                i_e_extensions,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=255"))]
    pub struct PDUSessionID(pub u8);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceAdmittedItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceAdmittedItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceAdmittedItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceAdmittedItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceAdmittedItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceAdmittedItemIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceAdmittedItemIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceAdmittedItemIEExtensions,
        AnonymousPDUSessionResourceAdmittedItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceAdmittedItem {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "handoverRequestAcknowledgeTransfer")]
        pub handover_request_acknowledge_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceAdmittedItemIEExtensions>,
    }
    impl PDUSessionResourceAdmittedItem {
        pub fn new(
            p_dusession_id: PDUSessionID,
            handover_request_acknowledge_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceAdmittedItemIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, handover_request_acknowledge_transfer, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceAdmittedList(pub SequenceOf<PDUSessionResourceAdmittedItem>);
    aper_sequence_of!(PDUSessionResourceAdmittedList, PDUSessionResourceAdmittedItem, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceFailedToModifyItemModCfmIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceFailedToModifyItemModCfmIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceFailedToModifyItemModCfmIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceFailedToModifyItemModCfmIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceFailedToModifyItemModCfmIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceFailedToModifyItemModCfmIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceFailedToModifyItemModCfmIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceFailedToModifyItemModCfmIEExtensions,
        AnonymousPDUSessionResourceFailedToModifyItemModCfmIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceFailedToModifyItemModCfm {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceModifyIndicationUnsuccessfulTransfer")]
        pub p_dusession_resource_modify_indication_unsuccessful_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceFailedToModifyItemModCfmIEExtensions>,
    }
    impl PDUSessionResourceFailedToModifyItemModCfm {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_modify_indication_unsuccessful_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceFailedToModifyItemModCfmIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_modify_indication_unsuccessful_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceFailedToModifyItemModResIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceFailedToModifyItemModResIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceFailedToModifyItemModResIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceFailedToModifyItemModResIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceFailedToModifyItemModResIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceFailedToModifyItemModResIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceFailedToModifyItemModResIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceFailedToModifyItemModResIEExtensions,
        AnonymousPDUSessionResourceFailedToModifyItemModResIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceFailedToModifyItemModRes {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceModifyUnsuccessfulTransfer")]
        pub p_dusession_resource_modify_unsuccessful_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceFailedToModifyItemModResIEExtensions>,
    }
    impl PDUSessionResourceFailedToModifyItemModRes {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_modify_unsuccessful_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceFailedToModifyItemModResIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_modify_unsuccessful_transfer, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceFailedToModifyListModCfm(pub SequenceOf<PDUSessionResourceFailedToModifyItemModCfm>);
    aper_sequence_of!(PDUSessionResourceFailedToModifyListModCfm, PDUSessionResourceFailedToModifyItemModCfm, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceFailedToModifyListModRes(pub SequenceOf<PDUSessionResourceFailedToModifyItemModRes>);
    aper_sequence_of!(PDUSessionResourceFailedToModifyListModRes, PDUSessionResourceFailedToModifyItemModRes, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceFailedToSetupItemCxtFailIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceFailedToSetupItemCxtFailIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceFailedToSetupItemCxtFailIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceFailedToSetupItemCxtFailIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceFailedToSetupItemCxtFailIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceFailedToSetupItemCxtFailIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceFailedToSetupItemCxtFailIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceFailedToSetupItemCxtFailIEExtensions,
        AnonymousPDUSessionResourceFailedToSetupItemCxtFailIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceFailedToSetupItemCxtFail {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceSetupUnsuccessfulTransfer")]
        pub p_dusession_resource_setup_unsuccessful_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceFailedToSetupItemCxtFailIEExtensions>,
    }
    impl PDUSessionResourceFailedToSetupItemCxtFail {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_setup_unsuccessful_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceFailedToSetupItemCxtFailIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_setup_unsuccessful_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceFailedToSetupItemCxtResIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceFailedToSetupItemCxtResIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceFailedToSetupItemCxtResIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceFailedToSetupItemCxtResIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceFailedToSetupItemCxtResIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceFailedToSetupItemCxtResIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceFailedToSetupItemCxtResIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceFailedToSetupItemCxtResIEExtensions,
        AnonymousPDUSessionResourceFailedToSetupItemCxtResIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceFailedToSetupItemCxtRes {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceSetupUnsuccessfulTransfer")]
        pub p_dusession_resource_setup_unsuccessful_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceFailedToSetupItemCxtResIEExtensions>,
    }
    impl PDUSessionResourceFailedToSetupItemCxtRes {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_setup_unsuccessful_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceFailedToSetupItemCxtResIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_setup_unsuccessful_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceFailedToSetupItemHOAckIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceFailedToSetupItemHOAckIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceFailedToSetupItemHOAckIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceFailedToSetupItemHOAckIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceFailedToSetupItemHOAckIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceFailedToSetupItemHOAckIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceFailedToSetupItemHOAckIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceFailedToSetupItemHOAckIEExtensions,
        AnonymousPDUSessionResourceFailedToSetupItemHOAckIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceFailedToSetupItemHOAck {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "handoverResourceAllocationUnsuccessfulTransfer")]
        pub handover_resource_allocation_unsuccessful_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceFailedToSetupItemHOAckIEExtensions>,
    }
    impl PDUSessionResourceFailedToSetupItemHOAck {
        pub fn new(
            p_dusession_id: PDUSessionID,
            handover_resource_allocation_unsuccessful_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceFailedToSetupItemHOAckIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, handover_resource_allocation_unsuccessful_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceFailedToSetupItemPSReqIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceFailedToSetupItemPSReqIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceFailedToSetupItemPSReqIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceFailedToSetupItemPSReqIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceFailedToSetupItemPSReqIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceFailedToSetupItemPSReqIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceFailedToSetupItemPSReqIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceFailedToSetupItemPSReqIEExtensions,
        AnonymousPDUSessionResourceFailedToSetupItemPSReqIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceFailedToSetupItemPSReq {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pathSwitchRequestSetupFailedTransfer")]
        pub path_switch_request_setup_failed_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceFailedToSetupItemPSReqIEExtensions>,
    }
    impl PDUSessionResourceFailedToSetupItemPSReq {
        pub fn new(
            p_dusession_id: PDUSessionID,
            path_switch_request_setup_failed_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceFailedToSetupItemPSReqIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, path_switch_request_setup_failed_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceFailedToSetupItemSUResIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceFailedToSetupItemSUResIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceFailedToSetupItemSUResIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceFailedToSetupItemSUResIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceFailedToSetupItemSUResIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceFailedToSetupItemSUResIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceFailedToSetupItemSUResIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceFailedToSetupItemSUResIEExtensions,
        AnonymousPDUSessionResourceFailedToSetupItemSUResIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceFailedToSetupItemSURes {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceSetupUnsuccessfulTransfer")]
        pub p_dusession_resource_setup_unsuccessful_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceFailedToSetupItemSUResIEExtensions>,
    }
    impl PDUSessionResourceFailedToSetupItemSURes {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_setup_unsuccessful_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceFailedToSetupItemSUResIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_setup_unsuccessful_transfer, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceFailedToSetupListCxtFail(pub SequenceOf<PDUSessionResourceFailedToSetupItemCxtFail>);
    aper_sequence_of!(PDUSessionResourceFailedToSetupListCxtFail, PDUSessionResourceFailedToSetupItemCxtFail, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceFailedToSetupListCxtRes(pub SequenceOf<PDUSessionResourceFailedToSetupItemCxtRes>);
    aper_sequence_of!(PDUSessionResourceFailedToSetupListCxtRes, PDUSessionResourceFailedToSetupItemCxtRes, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceFailedToSetupListHOAck(pub SequenceOf<PDUSessionResourceFailedToSetupItemHOAck>);
    aper_sequence_of!(PDUSessionResourceFailedToSetupListHOAck, PDUSessionResourceFailedToSetupItemHOAck, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceFailedToSetupListPSReq(pub SequenceOf<PDUSessionResourceFailedToSetupItemPSReq>);
    aper_sequence_of!(PDUSessionResourceFailedToSetupListPSReq, PDUSessionResourceFailedToSetupItemPSReq, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceFailedToSetupListSURes(pub SequenceOf<PDUSessionResourceFailedToSetupItemSURes>);
    aper_sequence_of!(PDUSessionResourceFailedToSetupListSURes, PDUSessionResourceFailedToSetupItemSURes, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceHandoverItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceHandoverItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceHandoverItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceHandoverItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceHandoverItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceHandoverItemIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceHandoverItemIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceHandoverItemIEExtensions,
        AnonymousPDUSessionResourceHandoverItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceHandoverItem {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "handoverCommandTransfer")]
        pub handover_command_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceHandoverItemIEExtensions>,
    }
    impl PDUSessionResourceHandoverItem {
        pub fn new(
            p_dusession_id: PDUSessionID,
            handover_command_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceHandoverItemIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, handover_command_transfer, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceHandoverList(pub SequenceOf<PDUSessionResourceHandoverItem>);
    aper_sequence_of!(PDUSessionResourceHandoverList, PDUSessionResourceHandoverItem, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceInformationItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceInformationItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceInformationItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceInformationItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceInformationItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceInformationItemIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceInformationItemIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceInformationItemIEExtensions,
        AnonymousPDUSessionResourceInformationItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceInformationItem {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(identifier = "qosFlowInformationList")]
        pub qos_flow_information_list: QosFlowInformationList,
        #[rasn(identifier = "dRBsToQosFlowsMappingList")]
        pub d_rbs_to_qos_flows_mapping_list: Option<DRBsToQosFlowsMappingList>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceInformationItemIEExtensions>,
    }
    impl PDUSessionResourceInformationItem {
        pub fn new(
            p_dusession_id: PDUSessionID,
            qos_flow_information_list: QosFlowInformationList,
            d_rbs_to_qos_flows_mapping_list: Option<DRBsToQosFlowsMappingList>,
            i_e_extensions: Option<PDUSessionResourceInformationItemIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, qos_flow_information_list, d_rbs_to_qos_flows_mapping_list, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceInformationList(pub SequenceOf<PDUSessionResourceInformationItem>);
    aper_sequence_of!(PDUSessionResourceInformationList, PDUSessionResourceInformationItem, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceItemCxtRelCplIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceItemCxtRelCplIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceItemCxtRelCplIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceItemCxtRelCplIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceItemCxtRelCplIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceItemCxtRelCplIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceItemCxtRelCplIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceItemCxtRelCplIEExtensions,
        AnonymousPDUSessionResourceItemCxtRelCplIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceItemCxtRelCpl {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceItemCxtRelCplIEExtensions>,
    }
    impl PDUSessionResourceItemCxtRelCpl {
        pub fn new(
            p_dusession_id: PDUSessionID,
            i_e_extensions: Option<PDUSessionResourceItemCxtRelCplIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceItemCxtRelReqIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceItemCxtRelReqIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceItemCxtRelReqIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceItemCxtRelReqIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceItemCxtRelReqIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceItemCxtRelReqIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceItemCxtRelReqIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceItemCxtRelReqIEExtensions,
        AnonymousPDUSessionResourceItemCxtRelReqIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceItemCxtRelReq {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceItemCxtRelReqIEExtensions>,
    }
    impl PDUSessionResourceItemCxtRelReq {
        pub fn new(
            p_dusession_id: PDUSessionID,
            i_e_extensions: Option<PDUSessionResourceItemCxtRelReqIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceItemHORqdIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceItemHORqdIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceItemHORqdIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceItemHORqdIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceItemHORqdIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceItemHORqdIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceItemHORqdIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceItemHORqdIEExtensions,
        AnonymousPDUSessionResourceItemHORqdIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceItemHORqd {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "handoverRequiredTransfer")]
        pub handover_required_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceItemHORqdIEExtensions>,
    }
    impl PDUSessionResourceItemHORqd {
        pub fn new(
            p_dusession_id: PDUSessionID,
            handover_required_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceItemHORqdIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, handover_required_transfer, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceListCxtRelCpl(pub SequenceOf<PDUSessionResourceItemCxtRelCpl>);
    aper_sequence_of!(PDUSessionResourceListCxtRelCpl, PDUSessionResourceItemCxtRelCpl, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceListCxtRelReq(pub SequenceOf<PDUSessionResourceItemCxtRelReq>);
    aper_sequence_of!(PDUSessionResourceListCxtRelReq, PDUSessionResourceItemCxtRelReq, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceListHORqd(pub SequenceOf<PDUSessionResourceItemHORqd>);
    aper_sequence_of!(PDUSessionResourceListHORqd, PDUSessionResourceItemHORqd, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyConfirmTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyConfirmTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyConfirmTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceModifyConfirmTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyConfirmTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceModifyConfirmTransferIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceModifyConfirmTransferIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyConfirmTransferIEExtensions,
        AnonymousPDUSessionResourceModifyConfirmTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyConfirmTransfer {
        #[rasn(identifier = "qosFlowModifyConfirmList")]
        pub qos_flow_modify_confirm_list: QosFlowModifyConfirmList,
        #[rasn(identifier = "uLNGU-UP-TNLInformation")]
        pub u_lngu_up_tnlinformation: UPTransportLayerInformation,
        #[rasn(identifier = "additionalNG-UUPTNLInformation")]
        pub additional_ng_uuptnlinformation: Option<UPTransportLayerInformationPairList>,
        #[rasn(identifier = "qosFlowFailedToModifyList")]
        pub qos_flow_failed_to_modify_list: Option<QosFlowListWithCause>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceModifyConfirmTransferIEExtensions>,
    }
    impl PDUSessionResourceModifyConfirmTransfer {
        pub fn new(
            qos_flow_modify_confirm_list: QosFlowModifyConfirmList,
            u_lngu_up_tnlinformation: UPTransportLayerInformation,
            additional_ng_uuptnlinformation: Option<UPTransportLayerInformationPairList>,
            qos_flow_failed_to_modify_list: Option<QosFlowListWithCause>,
            i_e_extensions: Option<PDUSessionResourceModifyConfirmTransferIEExtensions>,
        ) -> Self {
            Self {
                qos_flow_modify_confirm_list,
                u_lngu_up_tnlinformation,
                additional_ng_uuptnlinformation,
                qos_flow_failed_to_modify_list,
                i_e_extensions,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyIndicationTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyIndicationTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyIndicationTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceModifyIndicationTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyIndicationTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceModifyIndicationTransferIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceModifyIndicationTransferIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyIndicationTransferIEExtensions,
        AnonymousPDUSessionResourceModifyIndicationTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyIndicationTransfer {
        #[rasn(identifier = "dLQosFlowPerTNLInformation")]
        pub d_lqos_flow_per_tnlinformation: QosFlowPerTNLInformation,
        #[rasn(identifier = "additionalDLQosFlowPerTNLInformation")]
        pub additional_dlqos_flow_per_tnlinformation: Option<QosFlowPerTNLInformationList>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceModifyIndicationTransferIEExtensions>,
    }
    impl PDUSessionResourceModifyIndicationTransfer {
        pub fn new(
            d_lqos_flow_per_tnlinformation: QosFlowPerTNLInformation,
            additional_dlqos_flow_per_tnlinformation: Option<QosFlowPerTNLInformationList>,
            i_e_extensions: Option<PDUSessionResourceModifyIndicationTransferIEExtensions>,
        ) -> Self {
            Self { d_lqos_flow_per_tnlinformation, additional_dlqos_flow_per_tnlinformation, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyIndicationUnsuccessfulTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyIndicationUnsuccessfulTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyIndicationUnsuccessfulTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceModifyIndicationUnsuccessfulTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyIndicationUnsuccessfulTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceModifyIndicationUnsuccessfulTransferIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceModifyIndicationUnsuccessfulTransferIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyIndicationUnsuccessfulTransferIEExtensions,
        AnonymousPDUSessionResourceModifyIndicationUnsuccessfulTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyIndicationUnsuccessfulTransfer {
        pub cause: Cause,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceModifyIndicationUnsuccessfulTransferIEExtensions>,
    }
    impl PDUSessionResourceModifyIndicationUnsuccessfulTransfer {
        pub fn new(
            cause: Cause,
            i_e_extensions: Option<PDUSessionResourceModifyIndicationUnsuccessfulTransferIEExtensions>,
        ) -> Self {
            Self { cause, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyItemModCfmIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyItemModCfmIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyItemModCfmIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceModifyItemModCfmIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyItemModCfmIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceModifyItemModCfmIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceModifyItemModCfmIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyItemModCfmIEExtensions,
        AnonymousPDUSessionResourceModifyItemModCfmIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyItemModCfm {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceModifyConfirmTransfer")]
        pub p_dusession_resource_modify_confirm_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceModifyItemModCfmIEExtensions>,
    }
    impl PDUSessionResourceModifyItemModCfm {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_modify_confirm_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceModifyItemModCfmIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_modify_confirm_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyItemModIndIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyItemModIndIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyItemModIndIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceModifyItemModIndIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyItemModIndIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceModifyItemModIndIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceModifyItemModIndIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyItemModIndIEExtensions,
        AnonymousPDUSessionResourceModifyItemModIndIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyItemModInd {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceModifyIndicationTransfer")]
        pub p_dusession_resource_modify_indication_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceModifyItemModIndIEExtensions>,
    }
    impl PDUSessionResourceModifyItemModInd {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_modify_indication_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceModifyItemModIndIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_modify_indication_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyItemModReqIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyItemModReqIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyItemModReqIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceModifyItemModReqIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyItemModReqIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceModifyItemModReqIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceModifyItemModReqIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyItemModReqIEExtensions,
        AnonymousPDUSessionResourceModifyItemModReqIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyItemModReq {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(identifier = "nAS-PDU")]
        pub n_as_pdu: Option<NASPDU>,
        #[rasn(value("0.."), identifier = "pDUSessionResourceModifyRequestTransfer")]
        pub p_dusession_resource_modify_request_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceModifyItemModReqIEExtensions>,
    }
    impl PDUSessionResourceModifyItemModReq {
        pub fn new(
            p_dusession_id: PDUSessionID,
            n_as_pdu: Option<NASPDU>,
            p_dusession_resource_modify_request_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceModifyItemModReqIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, n_as_pdu, p_dusession_resource_modify_request_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyItemModResIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyItemModResIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyItemModResIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceModifyItemModResIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyItemModResIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceModifyItemModResIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceModifyItemModResIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyItemModResIEExtensions,
        AnonymousPDUSessionResourceModifyItemModResIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyItemModRes {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceModifyResponseTransfer")]
        pub p_dusession_resource_modify_response_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceModifyItemModResIEExtensions>,
    }
    impl PDUSessionResourceModifyItemModRes {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_modify_response_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceModifyItemModResIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_modify_response_transfer, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceModifyListModCfm(pub SequenceOf<PDUSessionResourceModifyItemModCfm>);
    aper_sequence_of!(PDUSessionResourceModifyListModCfm, PDUSessionResourceModifyItemModCfm, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceModifyListModInd(pub SequenceOf<PDUSessionResourceModifyItemModInd>);
    aper_sequence_of!(PDUSessionResourceModifyListModInd, PDUSessionResourceModifyItemModInd, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceModifyListModReq(pub SequenceOf<PDUSessionResourceModifyItemModReq>);
    aper_sequence_of!(PDUSessionResourceModifyListModReq, PDUSessionResourceModifyItemModReq, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceModifyListModRes(pub SequenceOf<PDUSessionResourceModifyItemModRes>);
    aper_sequence_of!(PDUSessionResourceModifyListModRes, PDUSessionResourceModifyItemModRes, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyRequestTransferProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyRequestTransferProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyRequestTransferProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPDUSessionResourceModifyRequestTransferProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyRequestTransferProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PDUSessionResourceModifyRequestTransferProtocolIEs(
        pub SequenceOf<AnonymousPDUSessionResourceModifyRequestTransferProtocolIEs>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyRequestTransferProtocolIEs,
        AnonymousPDUSessionResourceModifyRequestTransferProtocolIEs,
        0,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyRequestTransfer {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PDUSessionResourceModifyRequestTransferProtocolIEs,
    }
    impl PDUSessionResourceModifyRequestTransfer {
        pub fn new(protocol_ies: PDUSessionResourceModifyRequestTransferProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyResponseTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyResponseTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyResponseTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceModifyResponseTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyResponseTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceModifyResponseTransferIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceModifyResponseTransferIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyResponseTransferIEExtensions,
        AnonymousPDUSessionResourceModifyResponseTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyResponseTransfer {
        #[rasn(identifier = "dL-NGU-UP-TNLInformation")]
        pub d_l_ngu_up_tnlinformation: Option<UPTransportLayerInformation>,
        #[rasn(identifier = "uL-NGU-UP-TNLInformation")]
        pub u_l_ngu_up_tnlinformation: Option<UPTransportLayerInformation>,
        #[rasn(identifier = "qosFlowAddOrModifyResponseList")]
        pub qos_flow_add_or_modify_response_list: Option<QosFlowAddOrModifyResponseList>,
        #[rasn(identifier = "additionalDLQosFlowPerTNLInformation")]
        pub additional_dlqos_flow_per_tnlinformation: Option<QosFlowPerTNLInformationList>,
        #[rasn(identifier = "qosFlowFailedToAddOrModifyList")]
        pub qos_flow_failed_to_add_or_modify_list: Option<QosFlowListWithCause>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceModifyResponseTransferIEExtensions>,
    }
    impl PDUSessionResourceModifyResponseTransfer {
        pub fn new(
            d_l_ngu_up_tnlinformation: Option<UPTransportLayerInformation>,
            u_l_ngu_up_tnlinformation: Option<UPTransportLayerInformation>,
            qos_flow_add_or_modify_response_list: Option<QosFlowAddOrModifyResponseList>,
            additional_dlqos_flow_per_tnlinformation: Option<QosFlowPerTNLInformationList>,
            qos_flow_failed_to_add_or_modify_list: Option<QosFlowListWithCause>,
            i_e_extensions: Option<PDUSessionResourceModifyResponseTransferIEExtensions>,
        ) -> Self {
            Self {
                d_l_ngu_up_tnlinformation,
                u_l_ngu_up_tnlinformation,
                qos_flow_add_or_modify_response_list,
                additional_dlqos_flow_per_tnlinformation,
                qos_flow_failed_to_add_or_modify_list,
                i_e_extensions,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyUnsuccessfulTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyUnsuccessfulTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyUnsuccessfulTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceModifyUnsuccessfulTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyUnsuccessfulTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceModifyUnsuccessfulTransferIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceModifyUnsuccessfulTransferIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyUnsuccessfulTransferIEExtensions,
        AnonymousPDUSessionResourceModifyUnsuccessfulTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyUnsuccessfulTransfer {
        pub cause: Cause,
        #[rasn(identifier = "criticalityDiagnostics")]
        pub criticality_diagnostics: Option<CriticalityDiagnostics>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceModifyUnsuccessfulTransferIEExtensions>,
    }
    impl PDUSessionResourceModifyUnsuccessfulTransfer {
        pub fn new(
            cause: Cause,
            criticality_diagnostics: Option<CriticalityDiagnostics>,
            i_e_extensions: Option<PDUSessionResourceModifyUnsuccessfulTransferIEExtensions>,
        ) -> Self {
            Self { cause, criticality_diagnostics, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceNotifyItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceNotifyItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceNotifyItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceNotifyItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceNotifyItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceNotifyItemIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceNotifyItemIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceNotifyItemIEExtensions,
        AnonymousPDUSessionResourceNotifyItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceNotifyItem {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceNotifyTransfer")]
        pub p_dusession_resource_notify_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceNotifyItemIEExtensions>,
    }
    impl PDUSessionResourceNotifyItem {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_notify_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceNotifyItemIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_notify_transfer, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceNotifyList(pub SequenceOf<PDUSessionResourceNotifyItem>);
    aper_sequence_of!(PDUSessionResourceNotifyList, PDUSessionResourceNotifyItem, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceNotifyReleasedTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceNotifyReleasedTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceNotifyReleasedTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceNotifyReleasedTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceNotifyReleasedTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceNotifyReleasedTransferIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceNotifyReleasedTransferIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceNotifyReleasedTransferIEExtensions,
        AnonymousPDUSessionResourceNotifyReleasedTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceNotifyReleasedTransfer {
        pub cause: Cause,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceNotifyReleasedTransferIEExtensions>,
    }
    impl PDUSessionResourceNotifyReleasedTransfer {
        pub fn new(cause: Cause, i_e_extensions: Option<PDUSessionResourceNotifyReleasedTransferIEExtensions>) -> Self {
            Self { cause, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceNotifyTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceNotifyTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceNotifyTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceNotifyTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceNotifyTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceNotifyTransferIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceNotifyTransferIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceNotifyTransferIEExtensions,
        AnonymousPDUSessionResourceNotifyTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceNotifyTransfer {
        #[rasn(identifier = "qosFlowNotifyList")]
        pub qos_flow_notify_list: Option<QosFlowNotifyList>,
        #[rasn(identifier = "qosFlowReleasedList")]
        pub qos_flow_released_list: Option<QosFlowListWithCause>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceNotifyTransferIEExtensions>,
    }
    impl PDUSessionResourceNotifyTransfer {
        pub fn new(
            qos_flow_notify_list: Option<QosFlowNotifyList>,
            qos_flow_released_list: Option<QosFlowListWithCause>,
            i_e_extensions: Option<PDUSessionResourceNotifyTransferIEExtensions>,
        ) -> Self {
            Self { qos_flow_notify_list, qos_flow_released_list, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceReleaseCommandTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceReleaseCommandTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceReleaseCommandTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceReleaseCommandTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceReleaseCommandTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceReleaseCommandTransferIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceReleaseCommandTransferIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceReleaseCommandTransferIEExtensions,
        AnonymousPDUSessionResourceReleaseCommandTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceReleaseCommandTransfer {
        pub cause: Cause,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceReleaseCommandTransferIEExtensions>,
    }
    impl PDUSessionResourceReleaseCommandTransfer {
        pub fn new(cause: Cause, i_e_extensions: Option<PDUSessionResourceReleaseCommandTransferIEExtensions>) -> Self {
            Self { cause, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceReleaseResponseTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceReleaseResponseTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceReleaseResponseTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceReleaseResponseTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceReleaseResponseTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceReleaseResponseTransferIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceReleaseResponseTransferIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceReleaseResponseTransferIEExtensions,
        AnonymousPDUSessionResourceReleaseResponseTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceReleaseResponseTransfer {
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceReleaseResponseTransferIEExtensions>,
    }
    impl PDUSessionResourceReleaseResponseTransfer {
        pub fn new(i_e_extensions: Option<PDUSessionResourceReleaseResponseTransferIEExtensions>) -> Self {
            Self { i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceReleasedItemNotIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceReleasedItemNotIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceReleasedItemNotIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceReleasedItemNotIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceReleasedItemNotIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceReleasedItemNotIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceReleasedItemNotIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceReleasedItemNotIEExtensions,
        AnonymousPDUSessionResourceReleasedItemNotIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceReleasedItemNot {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceNotifyReleasedTransfer")]
        pub p_dusession_resource_notify_released_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceReleasedItemNotIEExtensions>,
    }
    impl PDUSessionResourceReleasedItemNot {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_notify_released_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceReleasedItemNotIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_notify_released_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceReleasedItemPSAckIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceReleasedItemPSAckIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceReleasedItemPSAckIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceReleasedItemPSAckIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceReleasedItemPSAckIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceReleasedItemPSAckIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceReleasedItemPSAckIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceReleasedItemPSAckIEExtensions,
        AnonymousPDUSessionResourceReleasedItemPSAckIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceReleasedItemPSAck {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pathSwitchRequestUnsuccessfulTransfer")]
        pub path_switch_request_unsuccessful_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceReleasedItemPSAckIEExtensions>,
    }
    impl PDUSessionResourceReleasedItemPSAck {
        pub fn new(
            p_dusession_id: PDUSessionID,
            path_switch_request_unsuccessful_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceReleasedItemPSAckIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, path_switch_request_unsuccessful_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceReleasedItemPSFailIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceReleasedItemPSFailIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceReleasedItemPSFailIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceReleasedItemPSFailIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceReleasedItemPSFailIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceReleasedItemPSFailIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceReleasedItemPSFailIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceReleasedItemPSFailIEExtensions,
        AnonymousPDUSessionResourceReleasedItemPSFailIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceReleasedItemPSFail {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pathSwitchRequestUnsuccessfulTransfer")]
        pub path_switch_request_unsuccessful_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceReleasedItemPSFailIEExtensions>,
    }
    impl PDUSessionResourceReleasedItemPSFail {
        pub fn new(
            p_dusession_id: PDUSessionID,
            path_switch_request_unsuccessful_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceReleasedItemPSFailIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, path_switch_request_unsuccessful_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceReleasedItemRelResIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceReleasedItemRelResIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceReleasedItemRelResIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceReleasedItemRelResIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceReleasedItemRelResIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceReleasedItemRelResIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceReleasedItemRelResIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceReleasedItemRelResIEExtensions,
        AnonymousPDUSessionResourceReleasedItemRelResIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceReleasedItemRelRes {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceReleaseResponseTransfer")]
        pub p_dusession_resource_release_response_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceReleasedItemRelResIEExtensions>,
    }
    impl PDUSessionResourceReleasedItemRelRes {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_release_response_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceReleasedItemRelResIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_release_response_transfer, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceReleasedListNot(pub SequenceOf<PDUSessionResourceReleasedItemNot>);
    aper_sequence_of!(PDUSessionResourceReleasedListNot, PDUSessionResourceReleasedItemNot, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceReleasedListPSAck(pub SequenceOf<PDUSessionResourceReleasedItemPSAck>);
    aper_sequence_of!(PDUSessionResourceReleasedListPSAck, PDUSessionResourceReleasedItemPSAck, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceReleasedListPSFail(pub SequenceOf<PDUSessionResourceReleasedItemPSFail>);
    aper_sequence_of!(PDUSessionResourceReleasedListPSFail, PDUSessionResourceReleasedItemPSFail, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceReleasedListRelRes(pub SequenceOf<PDUSessionResourceReleasedItemRelRes>);
    aper_sequence_of!(PDUSessionResourceReleasedListRelRes, PDUSessionResourceReleasedItemRelRes, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceSecondaryRATUsageItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceSecondaryRATUsageItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceSecondaryRATUsageItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceSecondaryRATUsageItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceSecondaryRATUsageItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceSecondaryRATUsageItemIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceSecondaryRATUsageItemIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceSecondaryRATUsageItemIEExtensions,
        AnonymousPDUSessionResourceSecondaryRATUsageItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceSecondaryRATUsageItem {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "secondaryRATDataUsageReportTransfer")]
        pub secondary_ratdata_usage_report_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceSecondaryRATUsageItemIEExtensions>,
    }
    impl PDUSessionResourceSecondaryRATUsageItem {
        pub fn new(
            p_dusession_id: PDUSessionID,
            secondary_ratdata_usage_report_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceSecondaryRATUsageItemIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, secondary_ratdata_usage_report_transfer, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceSecondaryRATUsageList(pub SequenceOf<PDUSessionResourceSecondaryRATUsageItem>);
    aper_sequence_of!(PDUSessionResourceSecondaryRATUsageList, PDUSessionResourceSecondaryRATUsageItem, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceSetupItemCxtReqIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceSetupItemCxtReqIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceSetupItemCxtReqIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceSetupItemCxtReqIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceSetupItemCxtReqIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceSetupItemCxtReqIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceSetupItemCxtReqIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceSetupItemCxtReqIEExtensions,
        AnonymousPDUSessionResourceSetupItemCxtReqIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceSetupItemCxtReq {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(identifier = "nAS-PDU")]
        pub n_as_pdu: Option<NASPDU>,
        #[rasn(identifier = "s-NSSAI")]
        pub s_nssai: SNSSAI,
        #[rasn(value("0.."), identifier = "pDUSessionResourceSetupRequestTransfer")]
        pub p_dusession_resource_setup_request_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceSetupItemCxtReqIEExtensions>,
    }
    impl PDUSessionResourceSetupItemCxtReq {
        pub fn new(
            p_dusession_id: PDUSessionID,
            n_as_pdu: Option<NASPDU>,
            s_nssai: SNSSAI,
            p_dusession_resource_setup_request_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceSetupItemCxtReqIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, n_as_pdu, s_nssai, p_dusession_resource_setup_request_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceSetupItemCxtResIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceSetupItemCxtResIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceSetupItemCxtResIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceSetupItemCxtResIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceSetupItemCxtResIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceSetupItemCxtResIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceSetupItemCxtResIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceSetupItemCxtResIEExtensions,
        AnonymousPDUSessionResourceSetupItemCxtResIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceSetupItemCxtRes {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceSetupResponseTransfer")]
        pub p_dusession_resource_setup_response_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceSetupItemCxtResIEExtensions>,
    }
    impl PDUSessionResourceSetupItemCxtRes {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_setup_response_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceSetupItemCxtResIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_setup_response_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceSetupItemHOReqIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceSetupItemHOReqIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceSetupItemHOReqIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceSetupItemHOReqIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceSetupItemHOReqIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceSetupItemHOReqIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceSetupItemHOReqIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceSetupItemHOReqIEExtensions,
        AnonymousPDUSessionResourceSetupItemHOReqIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceSetupItemHOReq {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(identifier = "s-NSSAI")]
        pub s_nssai: SNSSAI,
        #[rasn(value("0.."), identifier = "handoverRequestTransfer")]
        pub handover_request_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceSetupItemHOReqIEExtensions>,
    }
    impl PDUSessionResourceSetupItemHOReq {
        pub fn new(
            p_dusession_id: PDUSessionID,
            s_nssai: SNSSAI,
            handover_request_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceSetupItemHOReqIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, s_nssai, handover_request_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceSetupItemSUReqIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceSetupItemSUReqIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceSetupItemSUReqIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceSetupItemSUReqIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceSetupItemSUReqIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceSetupItemSUReqIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceSetupItemSUReqIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceSetupItemSUReqIEExtensions,
        AnonymousPDUSessionResourceSetupItemSUReqIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceSetupItemSUReq {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(identifier = "pDUSessionNAS-PDU")]
        pub p_dusession_nas_pdu: Option<NASPDU>,
        #[rasn(identifier = "s-NSSAI")]
        pub s_nssai: SNSSAI,
        #[rasn(value("0.."), identifier = "pDUSessionResourceSetupRequestTransfer")]
        pub p_dusession_resource_setup_request_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceSetupItemSUReqIEExtensions>,
    }
    impl PDUSessionResourceSetupItemSUReq {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_nas_pdu: Option<NASPDU>,
            s_nssai: SNSSAI,
            p_dusession_resource_setup_request_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceSetupItemSUReqIEExtensions>,
        ) -> Self {
            Self {
                p_dusession_id,
                p_dusession_nas_pdu,
                s_nssai,
                p_dusession_resource_setup_request_transfer,
                i_e_extensions,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceSetupItemSUResIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceSetupItemSUResIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceSetupItemSUResIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceSetupItemSUResIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceSetupItemSUResIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceSetupItemSUResIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceSetupItemSUResIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceSetupItemSUResIEExtensions,
        AnonymousPDUSessionResourceSetupItemSUResIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceSetupItemSURes {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceSetupResponseTransfer")]
        pub p_dusession_resource_setup_response_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceSetupItemSUResIEExtensions>,
    }
    impl PDUSessionResourceSetupItemSURes {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_setup_response_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceSetupItemSUResIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_setup_response_transfer, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceSetupListCxtReq(pub SequenceOf<PDUSessionResourceSetupItemCxtReq>);
    aper_sequence_of!(PDUSessionResourceSetupListCxtReq, PDUSessionResourceSetupItemCxtReq, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceSetupListCxtRes(pub SequenceOf<PDUSessionResourceSetupItemCxtRes>);
    aper_sequence_of!(PDUSessionResourceSetupListCxtRes, PDUSessionResourceSetupItemCxtRes, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceSetupListHOReq(pub SequenceOf<PDUSessionResourceSetupItemHOReq>);
    aper_sequence_of!(PDUSessionResourceSetupListHOReq, PDUSessionResourceSetupItemHOReq, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceSetupListSUReq(pub SequenceOf<PDUSessionResourceSetupItemSUReq>);
    aper_sequence_of!(PDUSessionResourceSetupListSUReq, PDUSessionResourceSetupItemSUReq, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceSetupListSURes(pub SequenceOf<PDUSessionResourceSetupItemSURes>);
    aper_sequence_of!(PDUSessionResourceSetupListSURes, PDUSessionResourceSetupItemSURes, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceSetupRequestTransferProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceSetupRequestTransferProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceSetupRequestTransferProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPDUSessionResourceSetupRequestTransferProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceSetupRequestTransferProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PDUSessionResourceSetupRequestTransferProtocolIEs(
        pub SequenceOf<AnonymousPDUSessionResourceSetupRequestTransferProtocolIEs>,
    );
    aper_sequence_of!(
        PDUSessionResourceSetupRequestTransferProtocolIEs,
        AnonymousPDUSessionResourceSetupRequestTransferProtocolIEs,
        0,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceSetupRequestTransfer {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PDUSessionResourceSetupRequestTransferProtocolIEs,
    }
    impl PDUSessionResourceSetupRequestTransfer {
        pub fn new(protocol_ies: PDUSessionResourceSetupRequestTransferProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceSetupResponseTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceSetupResponseTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceSetupResponseTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceSetupResponseTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceSetupResponseTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceSetupResponseTransferIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceSetupResponseTransferIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceSetupResponseTransferIEExtensions,
        AnonymousPDUSessionResourceSetupResponseTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceSetupResponseTransfer {
        #[rasn(identifier = "dLQosFlowPerTNLInformation")]
        pub d_lqos_flow_per_tnlinformation: QosFlowPerTNLInformation,
        #[rasn(identifier = "additionalDLQosFlowPerTNLInformation")]
        pub additional_dlqos_flow_per_tnlinformation: Option<QosFlowPerTNLInformationList>,
        #[rasn(identifier = "securityResult")]
        pub security_result: Option<SecurityResult>,
        #[rasn(identifier = "qosFlowFailedToSetupList")]
        pub qos_flow_failed_to_setup_list: Option<QosFlowListWithCause>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceSetupResponseTransferIEExtensions>,
    }
    impl PDUSessionResourceSetupResponseTransfer {
        pub fn new(
            d_lqos_flow_per_tnlinformation: QosFlowPerTNLInformation,
            additional_dlqos_flow_per_tnlinformation: Option<QosFlowPerTNLInformationList>,
            security_result: Option<SecurityResult>,
            qos_flow_failed_to_setup_list: Option<QosFlowListWithCause>,
            i_e_extensions: Option<PDUSessionResourceSetupResponseTransferIEExtensions>,
        ) -> Self {
            Self {
                d_lqos_flow_per_tnlinformation,
                additional_dlqos_flow_per_tnlinformation,
                security_result,
                qos_flow_failed_to_setup_list,
                i_e_extensions,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceSetupUnsuccessfulTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceSetupUnsuccessfulTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceSetupUnsuccessfulTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceSetupUnsuccessfulTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceSetupUnsuccessfulTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceSetupUnsuccessfulTransferIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceSetupUnsuccessfulTransferIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceSetupUnsuccessfulTransferIEExtensions,
        AnonymousPDUSessionResourceSetupUnsuccessfulTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceSetupUnsuccessfulTransfer {
        pub cause: Cause,
        #[rasn(identifier = "criticalityDiagnostics")]
        pub criticality_diagnostics: Option<CriticalityDiagnostics>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceSetupUnsuccessfulTransferIEExtensions>,
    }
    impl PDUSessionResourceSetupUnsuccessfulTransfer {
        pub fn new(
            cause: Cause,
            criticality_diagnostics: Option<CriticalityDiagnostics>,
            i_e_extensions: Option<PDUSessionResourceSetupUnsuccessfulTransferIEExtensions>,
        ) -> Self {
            Self { cause, criticality_diagnostics, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceSwitchedItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceSwitchedItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceSwitchedItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceSwitchedItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceSwitchedItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceSwitchedItemIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceSwitchedItemIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceSwitchedItemIEExtensions,
        AnonymousPDUSessionResourceSwitchedItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceSwitchedItem {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pathSwitchRequestAcknowledgeTransfer")]
        pub path_switch_request_acknowledge_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceSwitchedItemIEExtensions>,
    }
    impl PDUSessionResourceSwitchedItem {
        pub fn new(
            p_dusession_id: PDUSessionID,
            path_switch_request_acknowledge_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceSwitchedItemIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, path_switch_request_acknowledge_transfer, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceSwitchedList(pub SequenceOf<PDUSessionResourceSwitchedItem>);
    aper_sequence_of!(PDUSessionResourceSwitchedList, PDUSessionResourceSwitchedItem, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceToBeSwitchedDLItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceToBeSwitchedDLItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceToBeSwitchedDLItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceToBeSwitchedDLItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceToBeSwitchedDLItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceToBeSwitchedDLItemIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceToBeSwitchedDLItemIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceToBeSwitchedDLItemIEExtensions,
        AnonymousPDUSessionResourceToBeSwitchedDLItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceToBeSwitchedDLItem {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pathSwitchRequestTransfer")]
        pub path_switch_request_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceToBeSwitchedDLItemIEExtensions>,
    }
    impl PDUSessionResourceToBeSwitchedDLItem {
        pub fn new(
            p_dusession_id: PDUSessionID,
            path_switch_request_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceToBeSwitchedDLItemIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, path_switch_request_transfer, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceToBeSwitchedDLList(pub SequenceOf<PDUSessionResourceToBeSwitchedDLItem>);
    aper_sequence_of!(PDUSessionResourceToBeSwitchedDLList, PDUSessionResourceToBeSwitchedDLItem, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceToReleaseItemHOCmdIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceToReleaseItemHOCmdIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceToReleaseItemHOCmdIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceToReleaseItemHOCmdIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceToReleaseItemHOCmdIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceToReleaseItemHOCmdIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceToReleaseItemHOCmdIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceToReleaseItemHOCmdIEExtensions,
        AnonymousPDUSessionResourceToReleaseItemHOCmdIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceToReleaseItemHOCmd {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "handoverPreparationUnsuccessfulTransfer")]
        pub handover_preparation_unsuccessful_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceToReleaseItemHOCmdIEExtensions>,
    }
    impl PDUSessionResourceToReleaseItemHOCmd {
        pub fn new(
            p_dusession_id: PDUSessionID,
            handover_preparation_unsuccessful_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceToReleaseItemHOCmdIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, handover_preparation_unsuccessful_transfer, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceToReleaseItemRelCmdIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceToReleaseItemRelCmdIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceToReleaseItemRelCmdIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionResourceToReleaseItemRelCmdIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceToReleaseItemRelCmdIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionResourceToReleaseItemRelCmdIEExtensions(
        pub SequenceOf<AnonymousPDUSessionResourceToReleaseItemRelCmdIEExtensions>,
    );
    aper_sequence_of!(
        PDUSessionResourceToReleaseItemRelCmdIEExtensions,
        AnonymousPDUSessionResourceToReleaseItemRelCmdIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceToReleaseItemRelCmd {
        #[rasn(identifier = "pDUSessionID")]
        pub p_dusession_id: PDUSessionID,
        #[rasn(value("0.."), identifier = "pDUSessionResourceReleaseCommandTransfer")]
        pub p_dusession_resource_release_command_transfer: OctetString,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionResourceToReleaseItemRelCmdIEExtensions>,
    }
    impl PDUSessionResourceToReleaseItemRelCmd {
        pub fn new(
            p_dusession_id: PDUSessionID,
            p_dusession_resource_release_command_transfer: OctetString,
            i_e_extensions: Option<PDUSessionResourceToReleaseItemRelCmdIEExtensions>,
        ) -> Self {
            Self { p_dusession_id, p_dusession_resource_release_command_transfer, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceToReleaseListHOCmd(pub SequenceOf<PDUSessionResourceToReleaseItemHOCmd>);
    aper_sequence_of!(PDUSessionResourceToReleaseListHOCmd, PDUSessionResourceToReleaseItemHOCmd, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct PDUSessionResourceToReleaseListRelCmd(pub SequenceOf<PDUSessionResourceToReleaseItemRelCmd>);
    aper_sequence_of!(PDUSessionResourceToReleaseListRelCmd, PDUSessionResourceToReleaseItemRelCmd, 1, 256);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum PDUSessionType {
        ipv4 = 0,
        ipv6 = 1,
        ipv4v6 = 2,
        ethernet = 3,
        unstructured = 4,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum PDUSessionUsageReportRATType {
        nr = 0,
        eutra = 1,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionUsageReportIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionUsageReportIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionUsageReportIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPDUSessionUsageReportIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionUsageReportIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PDUSessionUsageReportIEExtensions(pub SequenceOf<AnonymousPDUSessionUsageReportIEExtensions>);
    aper_sequence_of!(PDUSessionUsageReportIEExtensions, AnonymousPDUSessionUsageReportIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionUsageReport {
        #[rasn(identifier = "rATType")]
        pub r_attype: PDUSessionUsageReportRATType,
        #[rasn(identifier = "pDUSessionTimedReportList")]
        pub p_dusession_timed_report_list: VolumeTimedReportList,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PDUSessionUsageReportIEExtensions>,
    }
    impl PDUSessionUsageReport {
        pub fn new(
            r_attype: PDUSessionUsageReportRATType,
            p_dusession_timed_report_list: VolumeTimedReportList,
            i_e_extensions: Option<PDUSessionUsageReportIEExtensions>,
        ) -> Self {
            Self { r_attype, p_dusession_timed_report_list, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct PLMNIdentity(pub FixedOctetString<3usize>);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPLMNSupportItemIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPLMNSupportItemIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPLMNSupportItemIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPLMNSupportItemIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPLMNSupportItemIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PLMNSupportItemIEExtensions(pub SequenceOf<AnonymousPLMNSupportItemIEExtensions>);
    aper_sequence_of!(PLMNSupportItemIEExtensions, AnonymousPLMNSupportItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PLMNSupportItem {
        #[rasn(identifier = "pLMNIdentity")]
        pub p_lmnidentity: PLMNIdentity,
        #[rasn(identifier = "sliceSupportList")]
        pub slice_support_list: SliceSupportList,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PLMNSupportItemIEExtensions>,
    }
    impl PLMNSupportItem {
        pub fn new(
            p_lmnidentity: PLMNIdentity,
            slice_support_list: SliceSupportList,
            i_e_extensions: Option<PLMNSupportItemIEExtensions>,
        ) -> Self {
            Self { p_lmnidentity, slice_support_list, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=12"))]
    pub struct PLMNSupportList(pub SequenceOf<PLMNSupportItem>);
    aper_sequence_of!(PLMNSupportList, PLMNSupportItem, 1, 12);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum PWSFailedCellIDListChoiceExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct PWSFailedCellIDListChoiceExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: PWSFailedCellIDListChoiceExtensionsCriticality,
        pub value: Any,
    }
    impl PWSFailedCellIDListChoiceExtensions {
        pub fn new(id: u16, criticality: PWSFailedCellIDListChoiceExtensionsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum PWSFailedCellIDList {
        #[rasn(identifier = "eUTRA-CGI-PWSFailedList")]
        eUTRA_CGI_PWSFailedList(EUTRACGIList),
        #[rasn(identifier = "nR-CGI-PWSFailedList")]
        nR_CGI_PWSFailedList(NRCGIList),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(PWSFailedCellIDListChoiceExtensions),
    }
    #[doc = " P"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=1023", extensible))]
    pub struct PacketDelayBudget(pub Integer);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPacketErrorRateIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPacketErrorRateIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPacketErrorRateIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPacketErrorRateIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPacketErrorRateIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PacketErrorRateIEExtensions(pub SequenceOf<AnonymousPacketErrorRateIEExtensions>);
    aper_sequence_of!(PacketErrorRateIEExtensions, AnonymousPacketErrorRateIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PacketErrorRate {
        #[rasn(value("0..=9", extensible), identifier = "pERScalar")]
        pub p_erscalar: Integer,
        #[rasn(value("0..=9", extensible), identifier = "pERExponent")]
        pub p_erexponent: Integer,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PacketErrorRateIEExtensions>,
    }
    impl PacketErrorRate {
        pub fn new(
            p_erscalar: Integer,
            p_erexponent: Integer,
            i_e_extensions: Option<PacketErrorRateIEExtensions>,
        ) -> Self {
            Self { p_erscalar, p_erexponent, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=1000", extensible))]
    pub struct PacketLossRate(pub Integer);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("1..=16", extensible))]
    pub struct PagingAttemptCount(pub Integer);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPagingAttemptInformationIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPagingAttemptInformationIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPagingAttemptInformationIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPagingAttemptInformationIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPagingAttemptInformationIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PagingAttemptInformationIEExtensions(pub SequenceOf<AnonymousPagingAttemptInformationIEExtensions>);
    aper_sequence_of!(PagingAttemptInformationIEExtensions, AnonymousPagingAttemptInformationIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PagingAttemptInformation {
        #[rasn(identifier = "pagingAttemptCount")]
        pub paging_attempt_count: PagingAttemptCount,
        #[rasn(identifier = "intendedNumberOfPagingAttempts")]
        pub intended_number_of_paging_attempts: IntendedNumberOfPagingAttempts,
        #[rasn(identifier = "nextPagingAreaScope")]
        pub next_paging_area_scope: Option<NextPagingAreaScope>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PagingAttemptInformationIEExtensions>,
    }
    impl PagingAttemptInformation {
        pub fn new(
            paging_attempt_count: PagingAttemptCount,
            intended_number_of_paging_attempts: IntendedNumberOfPagingAttempts,
            next_paging_area_scope: Option<NextPagingAreaScope>,
            i_e_extensions: Option<PagingAttemptInformationIEExtensions>,
        ) -> Self {
            Self { paging_attempt_count, intended_number_of_paging_attempts, next_paging_area_scope, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum PagingDRX {
        v32 = 0,
        v64 = 1,
        v128 = 2,
        v256 = 3,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum PagingOrigin {
        #[rasn(identifier = "non-3gpp")]
        non_3gpp = 0,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum PagingPriority {
        priolevel1 = 0,
        priolevel2 = 1,
        priolevel3 = 2,
        priolevel4 = 3,
        priolevel5 = 4,
        priolevel6 = 5,
        priolevel7 = 6,
        priolevel8 = 7,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPathSwitchRequestAcknowledgeTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPathSwitchRequestAcknowledgeTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPathSwitchRequestAcknowledgeTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPathSwitchRequestAcknowledgeTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPathSwitchRequestAcknowledgeTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PathSwitchRequestAcknowledgeTransferIEExtensions(
        pub SequenceOf<AnonymousPathSwitchRequestAcknowledgeTransferIEExtensions>,
    );
    aper_sequence_of!(
        PathSwitchRequestAcknowledgeTransferIEExtensions,
        AnonymousPathSwitchRequestAcknowledgeTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PathSwitchRequestAcknowledgeTransfer {
        #[rasn(identifier = "uL-NGU-UP-TNLInformation")]
        pub u_l_ngu_up_tnlinformation: Option<UPTransportLayerInformation>,
        #[rasn(identifier = "securityIndication")]
        pub security_indication: Option<SecurityIndication>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PathSwitchRequestAcknowledgeTransferIEExtensions>,
    }
    impl PathSwitchRequestAcknowledgeTransfer {
        pub fn new(
            u_l_ngu_up_tnlinformation: Option<UPTransportLayerInformation>,
            security_indication: Option<SecurityIndication>,
            i_e_extensions: Option<PathSwitchRequestAcknowledgeTransferIEExtensions>,
        ) -> Self {
            Self { u_l_ngu_up_tnlinformation, security_indication, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPathSwitchRequestSetupFailedTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPathSwitchRequestSetupFailedTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPathSwitchRequestSetupFailedTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPathSwitchRequestSetupFailedTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPathSwitchRequestSetupFailedTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PathSwitchRequestSetupFailedTransferIEExtensions(
        pub SequenceOf<AnonymousPathSwitchRequestSetupFailedTransferIEExtensions>,
    );
    aper_sequence_of!(
        PathSwitchRequestSetupFailedTransferIEExtensions,
        AnonymousPathSwitchRequestSetupFailedTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PathSwitchRequestSetupFailedTransfer {
        pub cause: Cause,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PathSwitchRequestSetupFailedTransferIEExtensions>,
    }
    impl PathSwitchRequestSetupFailedTransfer {
        pub fn new(cause: Cause, i_e_extensions: Option<PathSwitchRequestSetupFailedTransferIEExtensions>) -> Self {
            Self { cause, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPathSwitchRequestTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPathSwitchRequestTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPathSwitchRequestTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPathSwitchRequestTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPathSwitchRequestTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PathSwitchRequestTransferIEExtensions(pub SequenceOf<AnonymousPathSwitchRequestTransferIEExtensions>);
    aper_sequence_of!(PathSwitchRequestTransferIEExtensions, AnonymousPathSwitchRequestTransferIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PathSwitchRequestTransfer {
        #[rasn(identifier = "dL-NGU-UP-TNLInformation")]
        pub d_l_ngu_up_tnlinformation: UPTransportLayerInformation,
        #[rasn(identifier = "dL-NGU-TNLInformationReused")]
        pub d_l_ngu_tnlinformation_reused: Option<DLNGUTNLInformationReused>,
        #[rasn(identifier = "userPlaneSecurityInformation")]
        pub user_plane_security_information: Option<UserPlaneSecurityInformation>,
        #[rasn(identifier = "qosFlowAcceptedList")]
        pub qos_flow_accepted_list: QosFlowAcceptedList,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PathSwitchRequestTransferIEExtensions>,
    }
    impl PathSwitchRequestTransfer {
        pub fn new(
            d_l_ngu_up_tnlinformation: UPTransportLayerInformation,
            d_l_ngu_tnlinformation_reused: Option<DLNGUTNLInformationReused>,
            user_plane_security_information: Option<UserPlaneSecurityInformation>,
            qos_flow_accepted_list: QosFlowAcceptedList,
            i_e_extensions: Option<PathSwitchRequestTransferIEExtensions>,
        ) -> Self {
            Self {
                d_l_ngu_up_tnlinformation,
                d_l_ngu_tnlinformation_reused,
                user_plane_security_information,
                qos_flow_accepted_list,
                i_e_extensions,
            }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPathSwitchRequestUnsuccessfulTransferIEExtensionsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPathSwitchRequestUnsuccessfulTransferIEExtensions {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPathSwitchRequestUnsuccessfulTransferIEExtensionsCriticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousPathSwitchRequestUnsuccessfulTransferIEExtensions {
        pub fn new(
            id: u16,
            criticality: AnonymousPathSwitchRequestUnsuccessfulTransferIEExtensionsCriticality,
            extension_value: Any,
        ) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PathSwitchRequestUnsuccessfulTransferIEExtensions(
        pub SequenceOf<AnonymousPathSwitchRequestUnsuccessfulTransferIEExtensions>,
    );
    aper_sequence_of!(
        PathSwitchRequestUnsuccessfulTransferIEExtensions,
        AnonymousPathSwitchRequestUnsuccessfulTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PathSwitchRequestUnsuccessfulTransfer {
        pub cause: Cause,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<PathSwitchRequestUnsuccessfulTransferIEExtensions>,
    }
    impl PathSwitchRequestUnsuccessfulTransfer {
        pub fn new(cause: Cause, i_e_extensions: Option<PathSwitchRequestUnsuccessfulTransferIEExtensions>) -> Self {
            Self { cause, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct PeriodicRegistrationUpdateTimer(pub FixedBitString<8usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct PortNumber(pub FixedOctetString<2usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated, identifier = "Pre-emptionCapability")]
    #[non_exhaustive]
    pub enum PreEmptionCapability {
        #[rasn(identifier = "shall-not-trigger-pre-emption")]
        shall_not_trigger_pre_emption = 0,
        #[rasn(identifier = "may-trigger-pre-emption")]
        may_trigger_pre_emption = 1,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated, identifier = "Pre-emptionVulnerability")]
    #[non_exhaustive]
    pub enum PreEmptionVulnerability {
        #[rasn(identifier = "not-pre-emptable")]
        not_pre_emptable = 0,
        #[rasn(identifier = "pre-emptable")]
        pre_emptable = 1,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("1..=15"))]
    pub struct PriorityLevelARP(pub u8);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("1..=127", extensible))]
    pub struct PriorityLevelQos(pub Integer);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum QoSFlowsUsageReportItemRATType {
        nr = 0,
        eutra = 1,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQoSFlowsUsageReportItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQoSFlowsUsageReportItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QoSFlowsUsageReportItemIEExtensions(pub SequenceOf<AnonymousQoSFlowsUsageReportItemIEExtensions>);
    aper_sequence_of!(QoSFlowsUsageReportItemIEExtensions, AnonymousQoSFlowsUsageReportItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "QoSFlowsUsageReport-Item")]
    #[non_exhaustive]
    pub struct QoSFlowsUsageReportItem {
        #[rasn(identifier = "qosFlowIdentifier")]
        pub qos_flow_identifier: QosFlowIdentifier,
        #[rasn(identifier = "rATType")]
        pub r_attype: QoSFlowsUsageReportItemRATType,
        #[rasn(identifier = "qoSFlowsTimedReportList")]
        pub qo_sflows_timed_report_list: VolumeTimedReportList,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QoSFlowsUsageReportItemIEExtensions>,
    }
    impl QoSFlowsUsageReportItem {
        pub fn new(
            qos_flow_identifier: QosFlowIdentifier,
            r_attype: QoSFlowsUsageReportItemRATType,
            qo_sflows_timed_report_list: VolumeTimedReportList,
            i_e_extensions: Option<QoSFlowsUsageReportItemIEExtensions>,
        ) -> Self {
            Self { qos_flow_identifier, r_attype, qo_sflows_timed_report_list, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct QoSFlowsUsageReportList(pub SequenceOf<QoSFlowsUsageReportItem>);
    aper_sequence_of!(QoSFlowsUsageReportList, QoSFlowsUsageReportItem, 1, 64);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct QosCharacteristicsChoiceExtensions {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl QosCharacteristicsChoiceExtensions {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Q"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum QosCharacteristics {
        nonDynamic5QI(NonDynamic5QIDescriptor),
        dynamic5QI(Dynamic5QIDescriptor),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(QosCharacteristicsChoiceExtensions),
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowAcceptedItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowAcceptedItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowAcceptedItemIEExtensions(pub SequenceOf<AnonymousQosFlowAcceptedItemIEExtensions>);
    aper_sequence_of!(QosFlowAcceptedItemIEExtensions, AnonymousQosFlowAcceptedItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowAcceptedItem {
        #[rasn(identifier = "qosFlowIdentifier")]
        pub qos_flow_identifier: QosFlowIdentifier,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowAcceptedItemIEExtensions>,
    }
    impl QosFlowAcceptedItem {
        pub fn new(
            qos_flow_identifier: QosFlowIdentifier,
            i_e_extensions: Option<QosFlowAcceptedItemIEExtensions>,
        ) -> Self {
            Self { qos_flow_identifier, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct QosFlowAcceptedList(pub SequenceOf<QosFlowAcceptedItem>);
    aper_sequence_of!(QosFlowAcceptedList, QosFlowAcceptedItem, 1, 64);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowAddOrModifyRequestItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowAddOrModifyRequestItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowAddOrModifyRequestItemIEExtensions(
        pub SequenceOf<AnonymousQosFlowAddOrModifyRequestItemIEExtensions>,
    );
    aper_sequence_of!(
        QosFlowAddOrModifyRequestItemIEExtensions,
        AnonymousQosFlowAddOrModifyRequestItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowAddOrModifyRequestItem {
        #[rasn(identifier = "qosFlowIdentifier")]
        pub qos_flow_identifier: QosFlowIdentifier,
        #[rasn(identifier = "qosFlowLevelQosParameters")]
        pub qos_flow_level_qos_parameters: Option<QosFlowLevelQosParameters>,
        #[rasn(identifier = "e-RAB-ID")]
        pub e_rab_id: Option<ERABID>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowAddOrModifyRequestItemIEExtensions>,
    }
    impl QosFlowAddOrModifyRequestItem {
        pub fn new(
            qos_flow_identifier: QosFlowIdentifier,
            qos_flow_level_qos_parameters: Option<QosFlowLevelQosParameters>,
            e_rab_id: Option<ERABID>,
            i_e_extensions: Option<QosFlowAddOrModifyRequestItemIEExtensions>,
        ) -> Self {
            Self { qos_flow_identifier, qos_flow_level_qos_parameters, e_rab_id, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct QosFlowAddOrModifyRequestList(pub SequenceOf<QosFlowAddOrModifyRequestItem>);
    aper_sequence_of!(QosFlowAddOrModifyRequestList, QosFlowAddOrModifyRequestItem, 1, 64);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowAddOrModifyResponseItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowAddOrModifyResponseItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowAddOrModifyResponseItemIEExtensions(
        pub SequenceOf<AnonymousQosFlowAddOrModifyResponseItemIEExtensions>,
    );
    aper_sequence_of!(
        QosFlowAddOrModifyResponseItemIEExtensions,
        AnonymousQosFlowAddOrModifyResponseItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowAddOrModifyResponseItem {
        #[rasn(identifier = "qosFlowIdentifier")]
        pub qos_flow_identifier: QosFlowIdentifier,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowAddOrModifyResponseItemIEExtensions>,
    }
    impl QosFlowAddOrModifyResponseItem {
        pub fn new(
            qos_flow_identifier: QosFlowIdentifier,
            i_e_extensions: Option<QosFlowAddOrModifyResponseItemIEExtensions>,
        ) -> Self {
            Self { qos_flow_identifier, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct QosFlowAddOrModifyResponseList(pub SequenceOf<QosFlowAddOrModifyResponseItem>);
    aper_sequence_of!(QosFlowAddOrModifyResponseList, QosFlowAddOrModifyResponseItem, 1, 64);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=63", extensible))]
    pub struct QosFlowIdentifier(pub Integer);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowInformationItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowInformationItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowInformationItemIEExtensions(pub SequenceOf<AnonymousQosFlowInformationItemIEExtensions>);
    aper_sequence_of!(QosFlowInformationItemIEExtensions, AnonymousQosFlowInformationItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowInformationItem {
        #[rasn(identifier = "qosFlowIdentifier")]
        pub qos_flow_identifier: QosFlowIdentifier,
        #[rasn(identifier = "dLForwarding")]
        pub d_lforwarding: Option<DLForwarding>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowInformationItemIEExtensions>,
    }
    impl QosFlowInformationItem {
        pub fn new(
            qos_flow_identifier: QosFlowIdentifier,
            d_lforwarding: Option<DLForwarding>,
            i_e_extensions: Option<QosFlowInformationItemIEExtensions>,
        ) -> Self {
            Self { qos_flow_identifier, d_lforwarding, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct QosFlowInformationList(pub SequenceOf<QosFlowInformationItem>);
    aper_sequence_of!(QosFlowInformationList, QosFlowInformationItem, 1, 64);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowItemWithDataForwardingIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowItemWithDataForwardingIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowItemWithDataForwardingIEExtensions(
        pub SequenceOf<AnonymousQosFlowItemWithDataForwardingIEExtensions>,
    );
    aper_sequence_of!(
        QosFlowItemWithDataForwardingIEExtensions,
        AnonymousQosFlowItemWithDataForwardingIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowItemWithDataForwarding {
        #[rasn(identifier = "qosFlowIdentifier")]
        pub qos_flow_identifier: QosFlowIdentifier,
        #[rasn(identifier = "dataForwardingAccepted")]
        pub data_forwarding_accepted: Option<DataForwardingAccepted>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowItemWithDataForwardingIEExtensions>,
    }
    impl QosFlowItemWithDataForwarding {
        pub fn new(
            qos_flow_identifier: QosFlowIdentifier,
            data_forwarding_accepted: Option<DataForwardingAccepted>,
            i_e_extensions: Option<QosFlowItemWithDataForwardingIEExtensions>,
        ) -> Self {
            Self { qos_flow_identifier, data_forwarding_accepted, i_e_extensions }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowLevelQosParametersIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowLevelQosParametersIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowLevelQosParametersIEExtensions(pub SequenceOf<AnonymousQosFlowLevelQosParametersIEExtensions>);
    aper_sequence_of!(QosFlowLevelQosParametersIEExtensions, AnonymousQosFlowLevelQosParametersIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowLevelQosParameters {
        #[rasn(identifier = "qosCharacteristics")]
        pub qos_characteristics: QosCharacteristics,
        #[rasn(identifier = "allocationAndRetentionPriority")]
        pub allocation_and_retention_priority: AllocationAndRetentionPriority,
        #[rasn(identifier = "gBR-QosInformation")]
        pub g_br_qos_information: Option<GBRQosInformation>,
        #[rasn(identifier = "reflectiveQosAttribute")]
        pub reflective_qos_attribute: Option<ReflectiveQosAttribute>,
        #[rasn(identifier = "additionalQosFlowInformation")]
        pub additional_qos_flow_information: Option<AdditionalQosFlowInformation>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowLevelQosParametersIEExtensions>,
    }
    impl QosFlowLevelQosParameters {
        pub fn new(
            qos_characteristics: QosCharacteristics,
            allocation_and_retention_priority: AllocationAndRetentionPriority,
            g_br_qos_information: Option<GBRQosInformation>,
            reflective_qos_attribute: Option<ReflectiveQosAttribute>,
            additional_qos_flow_information: Option<AdditionalQosFlowInformation>,
            i_e_extensions: Option<QosFlowLevelQosParametersIEExtensions>,
        ) -> Self {
            Self {
                qos_characteristics,
                allocation_and_retention_priority,
                g_br_qos_information,
                reflective_qos_attribute,
                additional_qos_flow_information,
                i_e_extensions,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct QosFlowListWithCause(pub SequenceOf<QosFlowWithCauseItem>);
    aper_sequence_of!(QosFlowListWithCause, QosFlowWithCauseItem, 1, 64);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct QosFlowListWithDataForwarding(pub SequenceOf<QosFlowItemWithDataForwarding>);
    aper_sequence_of!(QosFlowListWithDataForwarding, QosFlowItemWithDataForwarding, 1, 64);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowModifyConfirmItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowModifyConfirmItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowModifyConfirmItemIEExtensions(pub SequenceOf<AnonymousQosFlowModifyConfirmItemIEExtensions>);
    aper_sequence_of!(QosFlowModifyConfirmItemIEExtensions, AnonymousQosFlowModifyConfirmItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowModifyConfirmItem {
        #[rasn(identifier = "qosFlowIdentifier")]
        pub qos_flow_identifier: QosFlowIdentifier,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowModifyConfirmItemIEExtensions>,
    }
    impl QosFlowModifyConfirmItem {
        pub fn new(
            qos_flow_identifier: QosFlowIdentifier,
            i_e_extensions: Option<QosFlowModifyConfirmItemIEExtensions>,
        ) -> Self {
            Self { qos_flow_identifier, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct QosFlowModifyConfirmList(pub SequenceOf<QosFlowModifyConfirmItem>);
    aper_sequence_of!(QosFlowModifyConfirmList, QosFlowModifyConfirmItem, 1, 64);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowNotifyItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowNotifyItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowNotifyItemIEExtensions(pub SequenceOf<AnonymousQosFlowNotifyItemIEExtensions>);
    aper_sequence_of!(QosFlowNotifyItemIEExtensions, AnonymousQosFlowNotifyItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowNotifyItem {
        #[rasn(identifier = "qosFlowIdentifier")]
        pub qos_flow_identifier: QosFlowIdentifier,
        #[rasn(identifier = "notificationCause")]
        pub notification_cause: NotificationCause,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowNotifyItemIEExtensions>,
    }
    impl QosFlowNotifyItem {
        pub fn new(
            qos_flow_identifier: QosFlowIdentifier,
            notification_cause: NotificationCause,
            i_e_extensions: Option<QosFlowNotifyItemIEExtensions>,
        ) -> Self {
            Self { qos_flow_identifier, notification_cause, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct QosFlowNotifyList(pub SequenceOf<QosFlowNotifyItem>);
    aper_sequence_of!(QosFlowNotifyList, QosFlowNotifyItem, 1, 64);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowPerTNLInformationIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowPerTNLInformationIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowPerTNLInformationIEExtensions(pub SequenceOf<AnonymousQosFlowPerTNLInformationIEExtensions>);
    aper_sequence_of!(QosFlowPerTNLInformationIEExtensions, AnonymousQosFlowPerTNLInformationIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowPerTNLInformation {
        #[rasn(identifier = "uPTransportLayerInformation")]
        pub u_ptransport_layer_information: UPTransportLayerInformation,
        #[rasn(identifier = "associatedQosFlowList")]
        pub associated_qos_flow_list: AssociatedQosFlowList,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowPerTNLInformationIEExtensions>,
    }
    impl QosFlowPerTNLInformation {
        pub fn new(
            u_ptransport_layer_information: UPTransportLayerInformation,
            associated_qos_flow_list: AssociatedQosFlowList,
            i_e_extensions: Option<QosFlowPerTNLInformationIEExtensions>,
        ) -> Self {
            Self { u_ptransport_layer_information, associated_qos_flow_list, i_e_extensions }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowPerTNLInformationItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowPerTNLInformationItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowPerTNLInformationItemIEExtensions(
        pub SequenceOf<AnonymousQosFlowPerTNLInformationItemIEExtensions>,
    );
    aper_sequence_of!(
        QosFlowPerTNLInformationItemIEExtensions,
        AnonymousQosFlowPerTNLInformationItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowPerTNLInformationItem {
        #[rasn(identifier = "qosFlowPerTNLInformation")]
        pub qos_flow_per_tnlinformation: QosFlowPerTNLInformation,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowPerTNLInformationItemIEExtensions>,
    }
    impl QosFlowPerTNLInformationItem {
        pub fn new(
            qos_flow_per_tnlinformation: QosFlowPerTNLInformation,
            i_e_extensions: Option<QosFlowPerTNLInformationItemIEExtensions>,
        ) -> Self {
            Self { qos_flow_per_tnlinformation, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=3"))]
    pub struct QosFlowPerTNLInformationList(pub SequenceOf<QosFlowPerTNLInformationItem>);
    aper_sequence_of!(QosFlowPerTNLInformationList, QosFlowPerTNLInformationItem, 1, 3);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowSetupRequestItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowSetupRequestItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowSetupRequestItemIEExtensions(pub SequenceOf<AnonymousQosFlowSetupRequestItemIEExtensions>);
    aper_sequence_of!(QosFlowSetupRequestItemIEExtensions, AnonymousQosFlowSetupRequestItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowSetupRequestItem {
        #[rasn(identifier = "qosFlowIdentifier")]
        pub qos_flow_identifier: QosFlowIdentifier,
        #[rasn(identifier = "qosFlowLevelQosParameters")]
        pub qos_flow_level_qos_parameters: QosFlowLevelQosParameters,
        #[rasn(identifier = "e-RAB-ID")]
        pub e_rab_id: Option<ERABID>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowSetupRequestItemIEExtensions>,
    }
    impl QosFlowSetupRequestItem {
        pub fn new(
            qos_flow_identifier: QosFlowIdentifier,
            qos_flow_level_qos_parameters: QosFlowLevelQosParameters,
            e_rab_id: Option<ERABID>,
            i_e_extensions: Option<QosFlowSetupRequestItemIEExtensions>,
        ) -> Self {
            Self { qos_flow_identifier, qos_flow_level_qos_parameters, e_rab_id, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct QosFlowSetupRequestList(pub SequenceOf<QosFlowSetupRequestItem>);
    aper_sequence_of!(QosFlowSetupRequestList, QosFlowSetupRequestItem, 1, 64);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowSetupResponseItemSUResIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowSetupResponseItemSUResIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowSetupResponseItemSUResIEExtensions(
        pub SequenceOf<AnonymousQosFlowSetupResponseItemSUResIEExtensions>,
    );
    aper_sequence_of!(
        QosFlowSetupResponseItemSUResIEExtensions,
        AnonymousQosFlowSetupResponseItemSUResIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowSetupResponseItemSURes {
        #[rasn(identifier = "qosFlowIdentifier")]
        pub qos_flow_identifier: QosFlowIdentifier,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowSetupResponseItemSUResIEExtensions>,
    }
    impl QosFlowSetupResponseItemSURes {
        pub fn new(
            qos_flow_identifier: QosFlowIdentifier,
            i_e_extensions: Option<QosFlowSetupResponseItemSUResIEExtensions>,
        ) -> Self {
            Self { qos_flow_identifier, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct QosFlowSetupResponseListSURes(pub SequenceOf<QosFlowSetupResponseItemSURes>);
    aper_sequence_of!(QosFlowSetupResponseListSURes, QosFlowSetupResponseItemSURes, 1, 64);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowToBeForwardedItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowToBeForwardedItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowToBeForwardedItemIEExtensions(pub SequenceOf<AnonymousQosFlowToBeForwardedItemIEExtensions>);
    aper_sequence_of!(QosFlowToBeForwardedItemIEExtensions, AnonymousQosFlowToBeForwardedItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowToBeForwardedItem {
        #[rasn(identifier = "qosFlowIdentifier")]
        pub qos_flow_identifier: QosFlowIdentifier,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowToBeForwardedItemIEExtensions>,
    }
    impl QosFlowToBeForwardedItem {
        pub fn new(
            qos_flow_identifier: QosFlowIdentifier,
            i_e_extensions: Option<QosFlowToBeForwardedItemIEExtensions>,
        ) -> Self {
            Self { qos_flow_identifier, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct QosFlowToBeForwardedList(pub SequenceOf<QosFlowToBeForwardedItem>);
    aper_sequence_of!(QosFlowToBeForwardedList, QosFlowToBeForwardedItem, 1, 64);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousQosFlowWithCauseItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousQosFlowWithCauseItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct QosFlowWithCauseItemIEExtensions(pub SequenceOf<AnonymousQosFlowWithCauseItemIEExtensions>);
    aper_sequence_of!(QosFlowWithCauseItemIEExtensions, AnonymousQosFlowWithCauseItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct QosFlowWithCauseItem {
        #[rasn(identifier = "qosFlowIdentifier")]
        pub qos_flow_identifier: QosFlowIdentifier,
        pub cause: Cause,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<QosFlowWithCauseItemIEExtensions>,
    }
    impl QosFlowWithCauseItem {
        pub fn new(
            qos_flow_identifier: QosFlowIdentifier,
            cause: Cause,
            i_e_extensions: Option<QosFlowWithCauseItemIEExtensions>,
        ) -> Self {
            Self { qos_flow_identifier, cause, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "RAN-UE-NGAP-ID", value("0..=4294967295"))]
    pub struct RANUENGAPID(pub u32);
    #[doc = " R"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=150", extensible))]
    pub struct RANNodeName(pub PrintableString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("1..=256"))]
    pub struct RANPagingPriority(pub u16);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousRANStatusTransferTransparentContainerIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousRANStatusTransferTransparentContainerIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct RANStatusTransferTransparentContainerIEExtensions(
        pub SequenceOf<AnonymousRANStatusTransferTransparentContainerIEExtensions>,
    );
    aper_sequence_of!(
        RANStatusTransferTransparentContainerIEExtensions,
        AnonymousRANStatusTransferTransparentContainerIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "RANStatusTransfer-TransparentContainer")]
    #[non_exhaustive]
    pub struct RANStatusTransferTransparentContainer {
        #[rasn(identifier = "dRBsSubjectToStatusTransferList")]
        pub d_rbs_subject_to_status_transfer_list: DRBsSubjectToStatusTransferList,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<RANStatusTransferTransparentContainerIEExtensions>,
    }
    impl RANStatusTransferTransparentContainer {
        pub fn new(
            d_rbs_subject_to_status_transfer_list: DRBsSubjectToStatusTransferList,
            i_e_extensions: Option<RANStatusTransferTransparentContainerIEExtensions>,
        ) -> Self {
            Self { d_rbs_subject_to_status_transfer_list, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("8", extensible))]
    pub struct RATRestrictionInformation(pub BitString);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"))]
    pub struct RATRestrictions(pub SequenceOf<RATRestrictionsItem>);
    aper_sequence_of!(RATRestrictions, RATRestrictionsItem, 1, 16);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousRATRestrictionsItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousRATRestrictionsItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct RATRestrictionsItemIEExtensions(pub SequenceOf<AnonymousRATRestrictionsItemIEExtensions>);
    aper_sequence_of!(RATRestrictionsItemIEExtensions, AnonymousRATRestrictionsItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "RATRestrictions-Item")]
    #[non_exhaustive]
    pub struct RATRestrictionsItem {
        #[rasn(identifier = "pLMNIdentity")]
        pub p_lmnidentity: PLMNIdentity,
        #[rasn(identifier = "rATRestrictionInformation")]
        pub r_atrestriction_information: RATRestrictionInformation,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<RATRestrictionsItemIEExtensions>,
    }
    impl RATRestrictionsItem {
        pub fn new(
            p_lmnidentity: PLMNIdentity,
            r_atrestriction_information: RATRestrictionInformation,
            i_e_extensions: Option<RATRestrictionsItemIEExtensions>,
        ) -> Self {
            Self { p_lmnidentity, r_atrestriction_information, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct RRCContainer(pub OctetString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum RRCEstablishmentCause {
        emergency = 0,
        highPriorityAccess = 1,
        #[rasn(identifier = "mt-Access")]
        mt_Access = 2,
        #[rasn(identifier = "mo-Signalling")]
        mo_Signalling = 3,
        #[rasn(identifier = "mo-Data")]
        mo_Data = 4,
        #[rasn(identifier = "mo-VoiceCall")]
        mo_VoiceCall = 5,
        #[rasn(identifier = "mo-VideoCall")]
        mo_VideoCall = 6,
        #[rasn(identifier = "mo-SMS")]
        mo_SMS = 7,
        #[rasn(identifier = "mps-PriorityAccess")]
        mps_PriorityAccess = 8,
        #[rasn(identifier = "mcs-PriorityAccess")]
        mcs_PriorityAccess = 9,
        #[rasn(extension_addition)]
        notAvailable = 10,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum RRCInactiveTransitionReportRequest {
        #[rasn(identifier = "subsequent-state-transition-report")]
        subsequent_state_transition_report = 0,
        #[rasn(identifier = "single-rrc-connected-state-report")]
        single_rrc_connected_state_report = 1,
        #[rasn(identifier = "cancel-report")]
        cancel_report = 2,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum RRCState {
        inactive = 0,
        connected = 1,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousRecommendedCellItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousRecommendedCellItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct RecommendedCellItemIEExtensions(pub SequenceOf<AnonymousRecommendedCellItemIEExtensions>);
    aper_sequence_of!(RecommendedCellItemIEExtensions, AnonymousRecommendedCellItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct RecommendedCellItem {
        #[rasn(identifier = "nGRAN-CGI")]
        pub n_gran_cgi: NGRANCGI,
        #[rasn(value("0..=4095"), identifier = "timeStayedInCell")]
        pub time_stayed_in_cell: Option<u16>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<RecommendedCellItemIEExtensions>,
    }
    impl RecommendedCellItem {
        pub fn new(
            n_gran_cgi: NGRANCGI,
            time_stayed_in_cell: Option<u16>,
            i_e_extensions: Option<RecommendedCellItemIEExtensions>,
        ) -> Self {
            Self { n_gran_cgi, time_stayed_in_cell, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"))]
    pub struct RecommendedCellList(pub SequenceOf<RecommendedCellItem>);
    aper_sequence_of!(RecommendedCellList, RecommendedCellItem, 1, 16);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousRecommendedCellsForPagingIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousRecommendedCellsForPagingIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct RecommendedCellsForPagingIEExtensions(pub SequenceOf<AnonymousRecommendedCellsForPagingIEExtensions>);
    aper_sequence_of!(RecommendedCellsForPagingIEExtensions, AnonymousRecommendedCellsForPagingIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct RecommendedCellsForPaging {
        #[rasn(identifier = "recommendedCellList")]
        pub recommended_cell_list: RecommendedCellList,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<RecommendedCellsForPagingIEExtensions>,
    }
    impl RecommendedCellsForPaging {
        pub fn new(
            recommended_cell_list: RecommendedCellList,
            i_e_extensions: Option<RecommendedCellsForPagingIEExtensions>,
        ) -> Self {
            Self { recommended_cell_list, i_e_extensions }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousRecommendedRANNodeItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousRecommendedRANNodeItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct RecommendedRANNodeItemIEExtensions(pub SequenceOf<AnonymousRecommendedRANNodeItemIEExtensions>);
    aper_sequence_of!(RecommendedRANNodeItemIEExtensions, AnonymousRecommendedRANNodeItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct RecommendedRANNodeItem {
        #[rasn(identifier = "aMFPagingTarget")]
        pub a_mfpaging_target: AMFPagingTarget,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<RecommendedRANNodeItemIEExtensions>,
    }
    impl RecommendedRANNodeItem {
        pub fn new(
            a_mfpaging_target: AMFPagingTarget,
            i_e_extensions: Option<RecommendedRANNodeItemIEExtensions>,
        ) -> Self {
            Self { a_mfpaging_target, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"))]
    pub struct RecommendedRANNodeList(pub SequenceOf<RecommendedRANNodeItem>);
    aper_sequence_of!(RecommendedRANNodeList, RecommendedRANNodeItem, 1, 16);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousRecommendedRANNodesForPagingIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousRecommendedRANNodesForPagingIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct RecommendedRANNodesForPagingIEExtensions(
        pub SequenceOf<AnonymousRecommendedRANNodesForPagingIEExtensions>,
    );
    aper_sequence_of!(
        RecommendedRANNodesForPagingIEExtensions,
        AnonymousRecommendedRANNodesForPagingIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct RecommendedRANNodesForPaging {
        #[rasn(identifier = "recommendedRANNodeList")]
        pub recommended_rannode_list: RecommendedRANNodeList,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<RecommendedRANNodesForPagingIEExtensions>,
    }
    impl RecommendedRANNodesForPaging {
        pub fn new(
            recommended_rannode_list: RecommendedRANNodeList,
            i_e_extensions: Option<RecommendedRANNodesForPagingIEExtensions>,
        ) -> Self {
            Self { recommended_rannode_list, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum RedirectionVoiceFallback {
        possible = 0,
        #[rasn(identifier = "not-possible")]
        not_possible = 1,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("1..=64", extensible))]
    pub struct ReferenceID(pub Integer);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum ReflectiveQosAttribute {
        #[rasn(identifier = "subject-to")]
        subject_to = 0,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct RejectedNSSAIinPLMN(pub FixedOctetString<32usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct RejectedNSSAIinTA(pub FixedOctetString<32usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=255"))]
    pub struct RelativeAMFCapacity(pub u8);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=131071"))]
    pub struct RepetitionPeriod(pub u32);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum ReportArea {
        cell = 0,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum ResetAll {
        #[rasn(identifier = "reset-all")]
        reset_all = 0,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct ResetTypeChoiceExtensions {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl ResetTypeChoiceExtensions {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum ResetType {
        #[rasn(identifier = "nG-Interface")]
        nG_Interface(ResetAll),
        #[rasn(identifier = "partOfNG-Interface")]
        partOfNG_Interface(UEAssociatedLogicalNGConnectionList),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(ResetTypeChoiceExtensions),
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct RoutingID(pub OctetString);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSNSSAIIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSNSSAIIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SNSSAIIEExtensions(pub SequenceOf<AnonymousSNSSAIIEExtensions>);
    aper_sequence_of!(SNSSAIIEExtensions, AnonymousSNSSAIIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "S-NSSAI")]
    #[non_exhaustive]
    pub struct SNSSAI {
        #[rasn(identifier = "sST")]
        pub s_st: SST,
        #[rasn(identifier = "sD")]
        pub s_d: Option<SD>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<SNSSAIIEExtensions>,
    }
    impl SNSSAI {
        pub fn new(s_st: SST, s_d: Option<SD>, i_e_extensions: Option<SNSSAIIEExtensions>) -> Self {
            Self { s_st, s_d, i_e_extensions }
        }
    }
    #[doc = " S"]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=2"), identifier = "SCTP-TLAs")]
    pub struct SCTPTLAs(pub SequenceOf<TransportLayerAddress>);
    aper_sequence_of!(SCTPTLAs, TransportLayerAddress, 1, 2);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct SD(pub FixedOctetString<3usize>);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSONConfigurationTransferIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSONConfigurationTransferIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SONConfigurationTransferIEExtensions(pub SequenceOf<AnonymousSONConfigurationTransferIEExtensions>);
    aper_sequence_of!(SONConfigurationTransferIEExtensions, AnonymousSONConfigurationTransferIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct SONConfigurationTransfer {
        #[rasn(identifier = "targetRANNodeID")]
        pub target_rannode_id: TargetRANNodeID,
        #[rasn(identifier = "sourceRANNodeID")]
        pub source_rannode_id: SourceRANNodeID,
        #[rasn(identifier = "sONInformation")]
        pub s_oninformation: SONInformation,
        #[rasn(identifier = "xnTNLConfigurationInfo")]
        pub xn_tnlconfiguration_info: Option<XnTNLConfigurationInfo>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<SONConfigurationTransferIEExtensions>,
    }
    impl SONConfigurationTransfer {
        pub fn new(
            target_rannode_id: TargetRANNodeID,
            source_rannode_id: SourceRANNodeID,
            s_oninformation: SONInformation,
            xn_tnlconfiguration_info: Option<XnTNLConfigurationInfo>,
            i_e_extensions: Option<SONConfigurationTransferIEExtensions>,
        ) -> Self {
            Self { target_rannode_id, source_rannode_id, s_oninformation, xn_tnlconfiguration_info, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct SONInformationChoiceExtensions {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl SONInformationChoiceExtensions {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum SONInformation {
        sONInformationRequest(SONInformationRequest),
        sONInformationReply(SONInformationReply),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(SONInformationChoiceExtensions),
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSONInformationReplyIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSONInformationReplyIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SONInformationReplyIEExtensions(pub SequenceOf<AnonymousSONInformationReplyIEExtensions>);
    aper_sequence_of!(SONInformationReplyIEExtensions, AnonymousSONInformationReplyIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct SONInformationReply {
        #[rasn(identifier = "xnTNLConfigurationInfo")]
        pub xn_tnlconfiguration_info: Option<XnTNLConfigurationInfo>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<SONInformationReplyIEExtensions>,
    }
    impl SONInformationReply {
        pub fn new(
            xn_tnlconfiguration_info: Option<XnTNLConfigurationInfo>,
            i_e_extensions: Option<SONInformationReplyIEExtensions>,
        ) -> Self {
            Self { xn_tnlconfiguration_info, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum SONInformationRequest {
        #[rasn(identifier = "xn-TNL-configuration-info")]
        xn_TNL_configuration_info = 0,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct SST(pub FixedOctetString<1usize>);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSecondaryRATDataUsageReportTransferIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSecondaryRATDataUsageReportTransferIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SecondaryRATDataUsageReportTransferIEExtensions(
        pub SequenceOf<AnonymousSecondaryRATDataUsageReportTransferIEExtensions>,
    );
    aper_sequence_of!(
        SecondaryRATDataUsageReportTransferIEExtensions,
        AnonymousSecondaryRATDataUsageReportTransferIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct SecondaryRATDataUsageReportTransfer {
        #[rasn(identifier = "secondaryRATUsageInformation")]
        pub secondary_ratusage_information: Option<SecondaryRATUsageInformation>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<SecondaryRATDataUsageReportTransferIEExtensions>,
    }
    impl SecondaryRATDataUsageReportTransfer {
        pub fn new(
            secondary_ratusage_information: Option<SecondaryRATUsageInformation>,
            i_e_extensions: Option<SecondaryRATDataUsageReportTransferIEExtensions>,
        ) -> Self {
            Self { secondary_ratusage_information, i_e_extensions }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSecondaryRATUsageInformationIEExtension {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSecondaryRATUsageInformationIEExtension {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SecondaryRATUsageInformationIEExtension(
        pub SequenceOf<AnonymousSecondaryRATUsageInformationIEExtension>,
    );
    aper_sequence_of!(
        SecondaryRATUsageInformationIEExtension,
        AnonymousSecondaryRATUsageInformationIEExtension,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct SecondaryRATUsageInformation {
        #[rasn(identifier = "pDUSessionUsageReport")]
        pub p_dusession_usage_report: Option<PDUSessionUsageReport>,
        #[rasn(identifier = "qosFlowsUsageReportList")]
        pub qos_flows_usage_report_list: Option<QoSFlowsUsageReportList>,
        #[rasn(identifier = "iE-Extension")]
        pub i_e_extension: Option<SecondaryRATUsageInformationIEExtension>,
    }
    impl SecondaryRATUsageInformation {
        pub fn new(
            p_dusession_usage_report: Option<PDUSessionUsageReport>,
            qos_flows_usage_report_list: Option<QoSFlowsUsageReportList>,
            i_e_extension: Option<SecondaryRATUsageInformationIEExtension>,
        ) -> Self {
            Self { p_dusession_usage_report, qos_flows_usage_report_list, i_e_extension }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSecurityContextIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSecurityContextIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SecurityContextIEExtensions(pub SequenceOf<AnonymousSecurityContextIEExtensions>);
    aper_sequence_of!(SecurityContextIEExtensions, AnonymousSecurityContextIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct SecurityContext {
        #[rasn(identifier = "nextHopChainingCount")]
        pub next_hop_chaining_count: NextHopChainingCount,
        #[rasn(identifier = "nextHopNH")]
        pub next_hop_nh: SecurityKey,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<SecurityContextIEExtensions>,
    }
    impl SecurityContext {
        pub fn new(
            next_hop_chaining_count: NextHopChainingCount,
            next_hop_nh: SecurityKey,
            i_e_extensions: Option<SecurityContextIEExtensions>,
        ) -> Self {
            Self { next_hop_chaining_count, next_hop_nh, i_e_extensions }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSecurityIndicationIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSecurityIndicationIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SecurityIndicationIEExtensions(pub SequenceOf<AnonymousSecurityIndicationIEExtensions>);
    aper_sequence_of!(SecurityIndicationIEExtensions, AnonymousSecurityIndicationIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct SecurityIndication {
        #[rasn(identifier = "integrityProtectionIndication")]
        pub integrity_protection_indication: IntegrityProtectionIndication,
        #[rasn(identifier = "confidentialityProtectionIndication")]
        pub confidentiality_protection_indication: ConfidentialityProtectionIndication,
        #[rasn(identifier = "maximumIntegrityProtectedDataRate-UL")]
        pub maximum_integrity_protected_data_rate_ul: Option<MaximumIntegrityProtectedDataRate>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<SecurityIndicationIEExtensions>,
    }
    impl SecurityIndication {
        pub fn new(
            integrity_protection_indication: IntegrityProtectionIndication,
            confidentiality_protection_indication: ConfidentialityProtectionIndication,
            maximum_integrity_protected_data_rate_ul: Option<MaximumIntegrityProtectedDataRate>,
            i_e_extensions: Option<SecurityIndicationIEExtensions>,
        ) -> Self {
            Self {
                integrity_protection_indication,
                confidentiality_protection_indication,
                maximum_integrity_protected_data_rate_ul,
                i_e_extensions,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct SecurityKey(pub FixedBitString<256usize>);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSecurityResultIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSecurityResultIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SecurityResultIEExtensions(pub SequenceOf<AnonymousSecurityResultIEExtensions>);
    aper_sequence_of!(SecurityResultIEExtensions, AnonymousSecurityResultIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct SecurityResult {
        #[rasn(identifier = "integrityProtectionResult")]
        pub integrity_protection_result: IntegrityProtectionResult,
        #[rasn(identifier = "confidentialityProtectionResult")]
        pub confidentiality_protection_result: ConfidentialityProtectionResult,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<SecurityResultIEExtensions>,
    }
    impl SecurityResult {
        pub fn new(
            integrity_protection_result: IntegrityProtectionResult,
            confidentiality_protection_result: ConfidentialityProtectionResult,
            i_e_extensions: Option<SecurityResultIEExtensions>,
        ) -> Self {
            Self { integrity_protection_result, confidentiality_protection_result, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct SerialNumber(pub FixedBitString<16usize>);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousServedGUAMIItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousServedGUAMIItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct ServedGUAMIItemIEExtensions(pub SequenceOf<AnonymousServedGUAMIItemIEExtensions>);
    aper_sequence_of!(ServedGUAMIItemIEExtensions, AnonymousServedGUAMIItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct ServedGUAMIItem {
        #[rasn(identifier = "gUAMI")]
        pub g_uami: GUAMI,
        #[rasn(identifier = "backupAMFName")]
        pub backup_amfname: Option<AMFName>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<ServedGUAMIItemIEExtensions>,
    }
    impl ServedGUAMIItem {
        pub fn new(
            g_uami: GUAMI,
            backup_amfname: Option<AMFName>,
            i_e_extensions: Option<ServedGUAMIItemIEExtensions>,
        ) -> Self {
            Self { g_uami, backup_amfname, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct ServedGUAMIList(pub SequenceOf<ServedGUAMIItem>);
    aper_sequence_of!(ServedGUAMIList, ServedGUAMIItem, 1, 256);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"))]
    pub struct ServiceAreaInformation(pub SequenceOf<ServiceAreaInformationItem>);
    aper_sequence_of!(ServiceAreaInformation, ServiceAreaInformationItem, 1, 16);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousServiceAreaInformationItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousServiceAreaInformationItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct ServiceAreaInformationItemIEExtensions(pub SequenceOf<AnonymousServiceAreaInformationItemIEExtensions>);
    aper_sequence_of!(
        ServiceAreaInformationItemIEExtensions,
        AnonymousServiceAreaInformationItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "ServiceAreaInformation-Item")]
    #[non_exhaustive]
    pub struct ServiceAreaInformationItem {
        #[rasn(identifier = "pLMNIdentity")]
        pub p_lmnidentity: PLMNIdentity,
        #[rasn(identifier = "allowedTACs")]
        pub allowed_tacs: Option<AllowedTACs>,
        #[rasn(identifier = "notAllowedTACs")]
        pub not_allowed_tacs: Option<NotAllowedTACs>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<ServiceAreaInformationItemIEExtensions>,
    }
    impl ServiceAreaInformationItem {
        pub fn new(
            p_lmnidentity: PLMNIdentity,
            allowed_tacs: Option<AllowedTACs>,
            not_allowed_tacs: Option<NotAllowedTACs>,
            i_e_extensions: Option<ServiceAreaInformationItemIEExtensions>,
        ) -> Self {
            Self { p_lmnidentity, allowed_tacs, not_allowed_tacs, i_e_extensions }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSliceOverloadItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSliceOverloadItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SliceOverloadItemIEExtensions(pub SequenceOf<AnonymousSliceOverloadItemIEExtensions>);
    aper_sequence_of!(SliceOverloadItemIEExtensions, AnonymousSliceOverloadItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct SliceOverloadItem {
        #[rasn(identifier = "s-NSSAI")]
        pub s_nssai: SNSSAI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<SliceOverloadItemIEExtensions>,
    }
    impl SliceOverloadItem {
        pub fn new(s_nssai: SNSSAI, i_e_extensions: Option<SliceOverloadItemIEExtensions>) -> Self {
            Self { s_nssai, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=1024"))]
    pub struct SliceOverloadList(pub SequenceOf<SliceOverloadItem>);
    aper_sequence_of!(SliceOverloadList, SliceOverloadItem, 1, 1024);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSliceSupportItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSliceSupportItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SliceSupportItemIEExtensions(pub SequenceOf<AnonymousSliceSupportItemIEExtensions>);
    aper_sequence_of!(SliceSupportItemIEExtensions, AnonymousSliceSupportItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct SliceSupportItem {
        #[rasn(identifier = "s-NSSAI")]
        pub s_nssai: SNSSAI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<SliceSupportItemIEExtensions>,
    }
    impl SliceSupportItem {
        pub fn new(s_nssai: SNSSAI, i_e_extensions: Option<SliceSupportItemIEExtensions>) -> Self {
            Self { s_nssai, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=1024"))]
    pub struct SliceSupportList(pub SequenceOf<SliceSupportItem>);
    aper_sequence_of!(SliceSupportList, SliceSupportItem, 1, 1024);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSourceNGRANNodeToTargetNGRANNodeTransparentContainerIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSourceNGRANNodeToTargetNGRANNodeTransparentContainerIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SourceNGRANNodeToTargetNGRANNodeTransparentContainerIEExtensions(
        pub SequenceOf<AnonymousSourceNGRANNodeToTargetNGRANNodeTransparentContainerIEExtensions>,
    );
    aper_sequence_of!(
        SourceNGRANNodeToTargetNGRANNodeTransparentContainerIEExtensions,
        AnonymousSourceNGRANNodeToTargetNGRANNodeTransparentContainerIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SourceNGRANNode-ToTargetNGRANNode-TransparentContainer")]
    #[non_exhaustive]
    pub struct SourceNGRANNodeToTargetNGRANNodeTransparentContainer {
        #[rasn(identifier = "rRCContainer")]
        pub r_rccontainer: RRCContainer,
        #[rasn(identifier = "pDUSessionResourceInformationList")]
        pub p_dusession_resource_information_list: Option<PDUSessionResourceInformationList>,
        #[rasn(identifier = "e-RABInformationList")]
        pub e_rabinformation_list: Option<ERABInformationList>,
        #[rasn(identifier = "targetCell-ID")]
        pub target_cell_id: NGRANCGI,
        #[rasn(identifier = "indexToRFSP")]
        pub index_to_rfsp: Option<IndexToRFSP>,
        #[rasn(identifier = "uEHistoryInformation")]
        pub u_ehistory_information: UEHistoryInformation,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<SourceNGRANNodeToTargetNGRANNodeTransparentContainerIEExtensions>,
    }
    impl SourceNGRANNodeToTargetNGRANNodeTransparentContainer {
        pub fn new(
            r_rccontainer: RRCContainer,
            p_dusession_resource_information_list: Option<PDUSessionResourceInformationList>,
            e_rabinformation_list: Option<ERABInformationList>,
            target_cell_id: NGRANCGI,
            index_to_rfsp: Option<IndexToRFSP>,
            u_ehistory_information: UEHistoryInformation,
            i_e_extensions: Option<SourceNGRANNodeToTargetNGRANNodeTransparentContainerIEExtensions>,
        ) -> Self {
            Self {
                r_rccontainer,
                p_dusession_resource_information_list,
                e_rabinformation_list,
                target_cell_id,
                index_to_rfsp,
                u_ehistory_information,
                i_e_extensions,
            }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum SourceOfUEActivityBehaviourInformation {
        #[rasn(identifier = "subscription-information")]
        subscription_information = 0,
        statistics = 1,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSourceRANNodeIDIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSourceRANNodeIDIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SourceRANNodeIDIEExtensions(pub SequenceOf<AnonymousSourceRANNodeIDIEExtensions>);
    aper_sequence_of!(SourceRANNodeIDIEExtensions, AnonymousSourceRANNodeIDIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct SourceRANNodeID {
        #[rasn(identifier = "globalRANNodeID")]
        pub global_rannode_id: GlobalRANNodeID,
        #[rasn(identifier = "selectedTAI")]
        pub selected_tai: TAI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<SourceRANNodeIDIEExtensions>,
    }
    impl SourceRANNodeID {
        pub fn new(
            global_rannode_id: GlobalRANNodeID,
            selected_tai: TAI,
            i_e_extensions: Option<SourceRANNodeIDIEExtensions>,
        ) -> Self {
            Self { global_rannode_id, selected_tai, i_e_extensions }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSourceToTargetAMFInformationRerouteIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSourceToTargetAMFInformationRerouteIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SourceToTargetAMFInformationRerouteIEExtensions(
        pub SequenceOf<AnonymousSourceToTargetAMFInformationRerouteIEExtensions>,
    );
    aper_sequence_of!(
        SourceToTargetAMFInformationRerouteIEExtensions,
        AnonymousSourceToTargetAMFInformationRerouteIEExtensions,
        1,
        65535
    );
    #[doc = " This IE includes a transparent container from the source RAN node to the target RAN node."]
    #[doc = " The octets of the OCTET STRING are encoded according to the specifications of the target system."]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SourceToTarget-AMFInformationReroute")]
    #[non_exhaustive]
    pub struct SourceToTargetAMFInformationReroute {
        #[rasn(identifier = "configuredNSSAI")]
        pub configured_nssai: Option<ConfiguredNSSAI>,
        #[rasn(identifier = "rejectedNSSAIinPLMN")]
        pub rejected_nssaiin_plmn: Option<RejectedNSSAIinPLMN>,
        #[rasn(identifier = "rejectedNSSAIinTA")]
        pub rejected_nssaiin_ta: Option<RejectedNSSAIinTA>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<SourceToTargetAMFInformationRerouteIEExtensions>,
    }
    impl SourceToTargetAMFInformationReroute {
        pub fn new(
            configured_nssai: Option<ConfiguredNSSAI>,
            rejected_nssaiin_plmn: Option<RejectedNSSAIinPLMN>,
            rejected_nssaiin_ta: Option<RejectedNSSAIinTA>,
            i_e_extensions: Option<SourceToTargetAMFInformationRerouteIEExtensions>,
        ) -> Self {
            Self { configured_nssai, rejected_nssaiin_plmn, rejected_nssaiin_ta, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "SourceToTarget-TransparentContainer")]
    pub struct SourceToTargetTransparentContainer(pub OctetString);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSupportedTAItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousSupportedTAItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct SupportedTAItemIEExtensions(pub SequenceOf<AnonymousSupportedTAItemIEExtensions>);
    aper_sequence_of!(SupportedTAItemIEExtensions, AnonymousSupportedTAItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct SupportedTAItem {
        #[rasn(identifier = "tAC")]
        pub t_ac: TAC,
        #[rasn(identifier = "broadcastPLMNList")]
        pub broadcast_plmnlist: BroadcastPLMNList,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<SupportedTAItemIEExtensions>,
    }
    impl SupportedTAItem {
        pub fn new(
            t_ac: TAC,
            broadcast_plmnlist: BroadcastPLMNList,
            i_e_extensions: Option<SupportedTAItemIEExtensions>,
        ) -> Self {
            Self { t_ac, broadcast_plmnlist, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct SupportedTAList(pub SequenceOf<SupportedTAItem>);
    aper_sequence_of!(SupportedTAList, SupportedTAItem, 1, 256);
    #[doc = " T"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct TAC(pub FixedOctetString<3usize>);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTAIIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousTAIIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TAIIEExtensions(pub SequenceOf<AnonymousTAIIEExtensions>);
    aper_sequence_of!(TAIIEExtensions, AnonymousTAIIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct TAI {
        #[rasn(identifier = "pLMNIdentity")]
        pub p_lmnidentity: PLMNIdentity,
        #[rasn(identifier = "tAC")]
        pub t_ac: TAC,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<TAIIEExtensions>,
    }
    impl TAI {
        pub fn new(p_lmnidentity: PLMNIdentity, t_ac: TAC, i_e_extensions: Option<TAIIEExtensions>) -> Self {
            Self { p_lmnidentity, t_ac, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TAIBroadcastEUTRA(pub SequenceOf<TAIBroadcastEUTRAItem>);
    aper_sequence_of!(TAIBroadcastEUTRA, TAIBroadcastEUTRAItem, 1, 65535);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTAIBroadcastEUTRAItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousTAIBroadcastEUTRAItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TAIBroadcastEUTRAItemIEExtensions(pub SequenceOf<AnonymousTAIBroadcastEUTRAItemIEExtensions>);
    aper_sequence_of!(TAIBroadcastEUTRAItemIEExtensions, AnonymousTAIBroadcastEUTRAItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "TAIBroadcastEUTRA-Item")]
    #[non_exhaustive]
    pub struct TAIBroadcastEUTRAItem {
        #[rasn(identifier = "tAI")]
        pub t_ai: TAI,
        #[rasn(identifier = "completedCellsInTAI-EUTRA")]
        pub completed_cells_in_tai_eutra: CompletedCellsInTAIEUTRA,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<TAIBroadcastEUTRAItemIEExtensions>,
    }
    impl TAIBroadcastEUTRAItem {
        pub fn new(
            t_ai: TAI,
            completed_cells_in_tai_eutra: CompletedCellsInTAIEUTRA,
            i_e_extensions: Option<TAIBroadcastEUTRAItemIEExtensions>,
        ) -> Self {
            Self { t_ai, completed_cells_in_tai_eutra, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TAIBroadcastNR(pub SequenceOf<TAIBroadcastNRItem>);
    aper_sequence_of!(TAIBroadcastNR, TAIBroadcastNRItem, 1, 65535);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTAIBroadcastNRItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousTAIBroadcastNRItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TAIBroadcastNRItemIEExtensions(pub SequenceOf<AnonymousTAIBroadcastNRItemIEExtensions>);
    aper_sequence_of!(TAIBroadcastNRItemIEExtensions, AnonymousTAIBroadcastNRItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "TAIBroadcastNR-Item")]
    #[non_exhaustive]
    pub struct TAIBroadcastNRItem {
        #[rasn(identifier = "tAI")]
        pub t_ai: TAI,
        #[rasn(identifier = "completedCellsInTAI-NR")]
        pub completed_cells_in_tai_nr: CompletedCellsInTAINR,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<TAIBroadcastNRItemIEExtensions>,
    }
    impl TAIBroadcastNRItem {
        pub fn new(
            t_ai: TAI,
            completed_cells_in_tai_nr: CompletedCellsInTAINR,
            i_e_extensions: Option<TAIBroadcastNRItemIEExtensions>,
        ) -> Self {
            Self { t_ai, completed_cells_in_tai_nr, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TAICancelledEUTRA(pub SequenceOf<TAICancelledEUTRAItem>);
    aper_sequence_of!(TAICancelledEUTRA, TAICancelledEUTRAItem, 1, 65535);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTAICancelledEUTRAItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousTAICancelledEUTRAItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TAICancelledEUTRAItemIEExtensions(pub SequenceOf<AnonymousTAICancelledEUTRAItemIEExtensions>);
    aper_sequence_of!(TAICancelledEUTRAItemIEExtensions, AnonymousTAICancelledEUTRAItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "TAICancelledEUTRA-Item")]
    #[non_exhaustive]
    pub struct TAICancelledEUTRAItem {
        #[rasn(identifier = "tAI")]
        pub t_ai: TAI,
        #[rasn(identifier = "cancelledCellsInTAI-EUTRA")]
        pub cancelled_cells_in_tai_eutra: CancelledCellsInTAIEUTRA,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<TAICancelledEUTRAItemIEExtensions>,
    }
    impl TAICancelledEUTRAItem {
        pub fn new(
            t_ai: TAI,
            cancelled_cells_in_tai_eutra: CancelledCellsInTAIEUTRA,
            i_e_extensions: Option<TAICancelledEUTRAItemIEExtensions>,
        ) -> Self {
            Self { t_ai, cancelled_cells_in_tai_eutra, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TAICancelledNR(pub SequenceOf<TAICancelledNRItem>);
    aper_sequence_of!(TAICancelledNR, TAICancelledNRItem, 1, 65535);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTAICancelledNRItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousTAICancelledNRItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TAICancelledNRItemIEExtensions(pub SequenceOf<AnonymousTAICancelledNRItemIEExtensions>);
    aper_sequence_of!(TAICancelledNRItemIEExtensions, AnonymousTAICancelledNRItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "TAICancelledNR-Item")]
    #[non_exhaustive]
    pub struct TAICancelledNRItem {
        #[rasn(identifier = "tAI")]
        pub t_ai: TAI,
        #[rasn(identifier = "cancelledCellsInTAI-NR")]
        pub cancelled_cells_in_tai_nr: CancelledCellsInTAINR,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<TAICancelledNRItemIEExtensions>,
    }
    impl TAICancelledNRItem {
        pub fn new(
            t_ai: TAI,
            cancelled_cells_in_tai_nr: CancelledCellsInTAINR,
            i_e_extensions: Option<TAICancelledNRItemIEExtensions>,
        ) -> Self {
            Self { t_ai, cancelled_cells_in_tai_nr, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"))]
    pub struct TAIListForInactive(pub SequenceOf<TAIListForInactiveItem>);
    aper_sequence_of!(TAIListForInactive, TAIListForInactiveItem, 1, 16);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTAIListForInactiveItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousTAIListForInactiveItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TAIListForInactiveItemIEExtensions(pub SequenceOf<AnonymousTAIListForInactiveItemIEExtensions>);
    aper_sequence_of!(TAIListForInactiveItemIEExtensions, AnonymousTAIListForInactiveItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct TAIListForInactiveItem {
        #[rasn(identifier = "tAI")]
        pub t_ai: TAI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<TAIListForInactiveItemIEExtensions>,
    }
    impl TAIListForInactiveItem {
        pub fn new(t_ai: TAI, i_e_extensions: Option<TAIListForInactiveItemIEExtensions>) -> Self {
            Self { t_ai, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"))]
    pub struct TAIListForPaging(pub SequenceOf<TAIListForPagingItem>);
    aper_sequence_of!(TAIListForPaging, TAIListForPagingItem, 1, 16);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTAIListForPagingItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousTAIListForPagingItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TAIListForPagingItemIEExtensions(pub SequenceOf<AnonymousTAIListForPagingItemIEExtensions>);
    aper_sequence_of!(TAIListForPagingItemIEExtensions, AnonymousTAIListForPagingItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct TAIListForPagingItem {
        #[rasn(identifier = "tAI")]
        pub t_ai: TAI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<TAIListForPagingItemIEExtensions>,
    }
    impl TAIListForPagingItem {
        pub fn new(t_ai: TAI, i_e_extensions: Option<TAIListForPagingItemIEExtensions>) -> Self {
            Self { t_ai, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=2048"))]
    pub struct TAIListForRestart(pub SequenceOf<TAI>);
    aper_sequence_of!(TAIListForRestart, TAI, 1, 2048);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TAIListForWarning(pub SequenceOf<TAI>);
    aper_sequence_of!(TAIListForWarning, TAI, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=255"))]
    pub struct TNLAddressWeightFactor(pub u8);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTNLAssociationItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousTNLAssociationItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TNLAssociationItemIEExtensions(pub SequenceOf<AnonymousTNLAssociationItemIEExtensions>);
    aper_sequence_of!(TNLAssociationItemIEExtensions, AnonymousTNLAssociationItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct TNLAssociationItem {
        #[rasn(identifier = "tNLAssociationAddress")]
        pub t_nlassociation_address: CPTransportLayerInformation,
        pub cause: Cause,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<TNLAssociationItemIEExtensions>,
    }
    impl TNLAssociationItem {
        pub fn new(
            t_nlassociation_address: CPTransportLayerInformation,
            cause: Cause,
            i_e_extensions: Option<TNLAssociationItemIEExtensions>,
        ) -> Self {
            Self { t_nlassociation_address, cause, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=32"))]
    pub struct TNLAssociationList(pub SequenceOf<TNLAssociationItem>);
    aper_sequence_of!(TNLAssociationList, TNLAssociationItem, 1, 32);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum TNLAssociationUsage {
        ue = 0,
        #[rasn(identifier = "non-ue")]
        non_ue = 1,
        both = 2,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct TargetIDChoiceExtensions {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl TargetIDChoiceExtensions {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum TargetID {
        targetRANNodeID(TargetRANNodeID),
        #[rasn(identifier = "targeteNB-ID")]
        targeteNB_ID(TargeteNBID),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(TargetIDChoiceExtensions),
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTargetNGRANNodeToSourceNGRANNodeTransparentContainerIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousTargetNGRANNodeToSourceNGRANNodeTransparentContainerIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TargetNGRANNodeToSourceNGRANNodeTransparentContainerIEExtensions(
        pub SequenceOf<AnonymousTargetNGRANNodeToSourceNGRANNodeTransparentContainerIEExtensions>,
    );
    aper_sequence_of!(
        TargetNGRANNodeToSourceNGRANNodeTransparentContainerIEExtensions,
        AnonymousTargetNGRANNodeToSourceNGRANNodeTransparentContainerIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "TargetNGRANNode-ToSourceNGRANNode-TransparentContainer")]
    #[non_exhaustive]
    pub struct TargetNGRANNodeToSourceNGRANNodeTransparentContainer {
        #[rasn(identifier = "rRCContainer")]
        pub r_rccontainer: RRCContainer,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<TargetNGRANNodeToSourceNGRANNodeTransparentContainerIEExtensions>,
    }
    impl TargetNGRANNodeToSourceNGRANNodeTransparentContainer {
        pub fn new(
            r_rccontainer: RRCContainer,
            i_e_extensions: Option<TargetNGRANNodeToSourceNGRANNodeTransparentContainerIEExtensions>,
        ) -> Self {
            Self { r_rccontainer, i_e_extensions }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTargetRANNodeIDIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousTargetRANNodeIDIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TargetRANNodeIDIEExtensions(pub SequenceOf<AnonymousTargetRANNodeIDIEExtensions>);
    aper_sequence_of!(TargetRANNodeIDIEExtensions, AnonymousTargetRANNodeIDIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct TargetRANNodeID {
        #[rasn(identifier = "globalRANNodeID")]
        pub global_rannode_id: GlobalRANNodeID,
        #[rasn(identifier = "selectedTAI")]
        pub selected_tai: TAI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<TargetRANNodeIDIEExtensions>,
    }
    impl TargetRANNodeID {
        pub fn new(
            global_rannode_id: GlobalRANNodeID,
            selected_tai: TAI,
            i_e_extensions: Option<TargetRANNodeIDIEExtensions>,
        ) -> Self {
            Self { global_rannode_id, selected_tai, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, identifier = "TargetToSource-TransparentContainer")]
    pub struct TargetToSourceTransparentContainer(pub OctetString);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTargeteNBIDIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousTargeteNBIDIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TargeteNBIDIEExtensions(pub SequenceOf<AnonymousTargeteNBIDIEExtensions>);
    aper_sequence_of!(TargeteNBIDIEExtensions, AnonymousTargeteNBIDIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "TargeteNB-ID")]
    #[non_exhaustive]
    pub struct TargeteNBID {
        #[rasn(identifier = "globalENB-ID")]
        pub global_enb_id: GlobalNgENBID,
        #[rasn(identifier = "selected-EPS-TAI")]
        pub selected_eps_tai: EPSTAI,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<TargeteNBIDIEExtensions>,
    }
    impl TargeteNBID {
        pub fn new(
            global_enb_id: GlobalNgENBID,
            selected_eps_tai: EPSTAI,
            i_e_extensions: Option<TargeteNBIDIEExtensions>,
        ) -> Self {
            Self { global_enb_id, selected_eps_tai, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct TimeStamp(pub FixedOctetString<4usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum TimeToWait {
        v1s = 0,
        v2s = 1,
        v5s = 2,
        v10s = 3,
        v20s = 4,
        v60s = 5,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=4095"))]
    pub struct TimeUEStayedInCell(pub u16);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("0..=40950"))]
    pub struct TimeUEStayedInCellEnhancedGranularity(pub u16);
    #[doc = " This IE includes a transparent container from the target RAN node to the source RAN node."]
    #[doc = " The octets of the OCTET STRING are encoded according to the specifications of the target system."]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum TimerApproachForGUAMIRemoval {
        #[rasn(identifier = "apply-timer")]
        apply_timer = 0,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTraceActivationIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousTraceActivationIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct TraceActivationIEExtensions(pub SequenceOf<AnonymousTraceActivationIEExtensions>);
    aper_sequence_of!(TraceActivationIEExtensions, AnonymousTraceActivationIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct TraceActivation {
        #[rasn(identifier = "nGRANTraceID")]
        pub n_grantrace_id: NGRANTraceID,
        #[rasn(identifier = "interfacesToTrace")]
        pub interfaces_to_trace: InterfacesToTrace,
        #[rasn(identifier = "traceDepth")]
        pub trace_depth: TraceDepth,
        #[rasn(identifier = "traceCollectionEntityIPAddress")]
        pub trace_collection_entity_ipaddress: TransportLayerAddress,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<TraceActivationIEExtensions>,
    }
    impl TraceActivation {
        pub fn new(
            n_grantrace_id: NGRANTraceID,
            interfaces_to_trace: InterfacesToTrace,
            trace_depth: TraceDepth,
            trace_collection_entity_ipaddress: TransportLayerAddress,
            i_e_extensions: Option<TraceActivationIEExtensions>,
        ) -> Self {
            Self { n_grantrace_id, interfaces_to_trace, trace_depth, trace_collection_entity_ipaddress, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum TraceDepth {
        minimum = 0,
        medium = 1,
        maximum = 2,
        minimumWithoutVendorSpecificExtension = 3,
        mediumWithoutVendorSpecificExtension = 4,
        maximumWithoutVendorSpecificExtension = 5,
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, value("1..=99"))]
    pub struct TrafficLoadReductionIndication(pub u8);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=160", extensible))]
    pub struct TransportLayerAddress(pub BitString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum TypeOfError {
        #[rasn(identifier = "not-understood")]
        not_understood = 0,
        missing = 1,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUENGAPIDPairIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousUENGAPIDPairIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct UENGAPIDPairIEExtensions(pub SequenceOf<AnonymousUENGAPIDPairIEExtensions>);
    aper_sequence_of!(UENGAPIDPairIEExtensions, AnonymousUENGAPIDPairIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "UE-NGAP-ID-pair")]
    #[non_exhaustive]
    pub struct UENGAPIDPair {
        #[rasn(identifier = "aMF-UE-NGAP-ID")]
        pub a_mf_ue_ngap_id: AMFUENGAPID,
        #[rasn(identifier = "rAN-UE-NGAP-ID")]
        pub r_an_ue_ngap_id: RANUENGAPID,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<UENGAPIDPairIEExtensions>,
    }
    impl UENGAPIDPair {
        pub fn new(
            a_mf_ue_ngap_id: AMFUENGAPID,
            r_an_ue_ngap_id: RANUENGAPID,
            i_e_extensions: Option<UENGAPIDPairIEExtensions>,
        ) -> Self {
            Self { a_mf_ue_ngap_id, r_an_ue_ngap_id, i_e_extensions }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct UENGAPIDsChoiceExtensions {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl UENGAPIDsChoiceExtensions {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags, identifier = "UE-NGAP-IDs")]
    pub enum UENGAPIDs {
        #[rasn(identifier = "uE-NGAP-ID-pair")]
        uE_NGAP_ID_pair(UENGAPIDPair),
        #[rasn(identifier = "aMF-UE-NGAP-ID")]
        aMF_UE_NGAP_ID(AMFUENGAPID),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(UENGAPIDsChoiceExtensions),
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUEAssociatedLogicalNGConnectionItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousUEAssociatedLogicalNGConnectionItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct UEAssociatedLogicalNGConnectionItemIEExtensions(
        pub SequenceOf<AnonymousUEAssociatedLogicalNGConnectionItemIEExtensions>,
    );
    aper_sequence_of!(
        UEAssociatedLogicalNGConnectionItemIEExtensions,
        AnonymousUEAssociatedLogicalNGConnectionItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "UE-associatedLogicalNG-connectionItem")]
    #[non_exhaustive]
    pub struct UEAssociatedLogicalNGConnectionItem {
        #[rasn(identifier = "aMF-UE-NGAP-ID")]
        pub a_mf_ue_ngap_id: Option<AMFUENGAPID>,
        #[rasn(identifier = "rAN-UE-NGAP-ID")]
        pub r_an_ue_ngap_id: Option<RANUENGAPID>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<UEAssociatedLogicalNGConnectionItemIEExtensions>,
    }
    impl UEAssociatedLogicalNGConnectionItem {
        pub fn new(
            a_mf_ue_ngap_id: Option<AMFUENGAPID>,
            r_an_ue_ngap_id: Option<RANUENGAPID>,
            i_e_extensions: Option<UEAssociatedLogicalNGConnectionItemIEExtensions>,
        ) -> Self {
            Self { a_mf_ue_ngap_id, r_an_ue_ngap_id, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65536"), identifier = "UE-associatedLogicalNG-connectionList")]
    pub struct UEAssociatedLogicalNGConnectionList(pub SequenceOf<UEAssociatedLogicalNGConnectionItem>);
    aper_sequence_of!(UEAssociatedLogicalNGConnectionList, UEAssociatedLogicalNGConnectionItem, 1, 65536);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUEAggregateMaximumBitRateIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousUEAggregateMaximumBitRateIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct UEAggregateMaximumBitRateIEExtensions(pub SequenceOf<AnonymousUEAggregateMaximumBitRateIEExtensions>);
    aper_sequence_of!(UEAggregateMaximumBitRateIEExtensions, AnonymousUEAggregateMaximumBitRateIEExtensions, 1, 65535);
    #[doc = " U"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UEAggregateMaximumBitRate {
        #[rasn(identifier = "uEAggregateMaximumBitRateDL")]
        pub u_eaggregate_maximum_bit_rate_dl: BitRate,
        #[rasn(identifier = "uEAggregateMaximumBitRateUL")]
        pub u_eaggregate_maximum_bit_rate_ul: BitRate,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<UEAggregateMaximumBitRateIEExtensions>,
    }
    impl UEAggregateMaximumBitRate {
        pub fn new(
            u_eaggregate_maximum_bit_rate_dl: BitRate,
            u_eaggregate_maximum_bit_rate_ul: BitRate,
            i_e_extensions: Option<UEAggregateMaximumBitRateIEExtensions>,
        ) -> Self {
            Self { u_eaggregate_maximum_bit_rate_dl, u_eaggregate_maximum_bit_rate_ul, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum UEContextRequest {
        requested = 0,
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"))]
    pub struct UEHistoryInformation(pub SequenceOf<LastVisitedCellItem>);
    aper_sequence_of!(UEHistoryInformation, LastVisitedCellItem, 1, 16);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct UEIdentityIndexValueChoiceExtensions {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl UEIdentityIndexValueChoiceExtensions {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum UEIdentityIndexValue {
        #[rasn(size("10"))]
        indexLength10(BitString),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(UEIdentityIndexValueChoiceExtensions),
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct UEPagingIdentityChoiceExtensions {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl UEPagingIdentityChoiceExtensions {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum UEPagingIdentity {
        #[rasn(identifier = "fiveG-S-TMSI")]
        fiveG_S_TMSI(FiveGSTMSI),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(UEPagingIdentityChoiceExtensions),
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum UEPresence {
        #[rasn(identifier = "in")]
        R_in = 0,
        out = 1,
        unknown = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUEPresenceInAreaOfInterestItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousUEPresenceInAreaOfInterestItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct UEPresenceInAreaOfInterestItemIEExtensions(
        pub SequenceOf<AnonymousUEPresenceInAreaOfInterestItemIEExtensions>,
    );
    aper_sequence_of!(
        UEPresenceInAreaOfInterestItemIEExtensions,
        AnonymousUEPresenceInAreaOfInterestItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UEPresenceInAreaOfInterestItem {
        #[rasn(identifier = "locationReportingReferenceID")]
        pub location_reporting_reference_id: LocationReportingReferenceID,
        #[rasn(identifier = "uEPresence")]
        pub u_epresence: UEPresence,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<UEPresenceInAreaOfInterestItemIEExtensions>,
    }
    impl UEPresenceInAreaOfInterestItem {
        pub fn new(
            location_reporting_reference_id: LocationReportingReferenceID,
            u_epresence: UEPresence,
            i_e_extensions: Option<UEPresenceInAreaOfInterestItemIEExtensions>,
        ) -> Self {
            Self { location_reporting_reference_id, u_epresence, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=64"))]
    pub struct UEPresenceInAreaOfInterestList(pub SequenceOf<UEPresenceInAreaOfInterestItem>);
    aper_sequence_of!(UEPresenceInAreaOfInterestList, UEPresenceInAreaOfInterestItem, 1, 64);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct UERadioCapability(pub OctetString);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUERadioCapabilityForPagingIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousUERadioCapabilityForPagingIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct UERadioCapabilityForPagingIEExtensions(pub SequenceOf<AnonymousUERadioCapabilityForPagingIEExtensions>);
    aper_sequence_of!(
        UERadioCapabilityForPagingIEExtensions,
        AnonymousUERadioCapabilityForPagingIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UERadioCapabilityForPaging {
        #[rasn(identifier = "uERadioCapabilityForPagingOfNR")]
        pub u_eradio_capability_for_paging_of_nr: Option<UERadioCapabilityForPagingOfNR>,
        #[rasn(identifier = "uERadioCapabilityForPagingOfEUTRA")]
        pub u_eradio_capability_for_paging_of_eutra: Option<UERadioCapabilityForPagingOfEUTRA>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<UERadioCapabilityForPagingIEExtensions>,
    }
    impl UERadioCapabilityForPaging {
        pub fn new(
            u_eradio_capability_for_paging_of_nr: Option<UERadioCapabilityForPagingOfNR>,
            u_eradio_capability_for_paging_of_eutra: Option<UERadioCapabilityForPagingOfEUTRA>,
            i_e_extensions: Option<UERadioCapabilityForPagingIEExtensions>,
        ) -> Self {
            Self { u_eradio_capability_for_paging_of_nr, u_eradio_capability_for_paging_of_eutra, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct UERadioCapabilityForPagingOfEUTRA(pub OctetString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct UERadioCapabilityForPagingOfNR(pub OctetString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum UERetentionInformation {
        #[rasn(identifier = "ues-retained")]
        ues_retained = 0,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUESecurityCapabilitiesIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousUESecurityCapabilitiesIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct UESecurityCapabilitiesIEExtensions(pub SequenceOf<AnonymousUESecurityCapabilitiesIEExtensions>);
    aper_sequence_of!(UESecurityCapabilitiesIEExtensions, AnonymousUESecurityCapabilitiesIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UESecurityCapabilities {
        #[rasn(identifier = "nRencryptionAlgorithms")]
        pub n_rencryption_algorithms: NRencryptionAlgorithms,
        #[rasn(identifier = "nRintegrityProtectionAlgorithms")]
        pub n_rintegrity_protection_algorithms: NRintegrityProtectionAlgorithms,
        #[rasn(identifier = "eUTRAencryptionAlgorithms")]
        pub e_utraencryption_algorithms: EUTRAencryptionAlgorithms,
        #[rasn(identifier = "eUTRAintegrityProtectionAlgorithms")]
        pub e_utraintegrity_protection_algorithms: EUTRAintegrityProtectionAlgorithms,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<UESecurityCapabilitiesIEExtensions>,
    }
    impl UESecurityCapabilities {
        pub fn new(
            n_rencryption_algorithms: NRencryptionAlgorithms,
            n_rintegrity_protection_algorithms: NRintegrityProtectionAlgorithms,
            e_utraencryption_algorithms: EUTRAencryptionAlgorithms,
            e_utraintegrity_protection_algorithms: EUTRAintegrityProtectionAlgorithms,
            i_e_extensions: Option<UESecurityCapabilitiesIEExtensions>,
        ) -> Self {
            Self {
                n_rencryption_algorithms,
                n_rintegrity_protection_algorithms,
                e_utraencryption_algorithms,
                e_utraintegrity_protection_algorithms,
                i_e_extensions,
            }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousULNGUUPTNLModifyItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousULNGUUPTNLModifyItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct ULNGUUPTNLModifyItemIEExtensions(pub SequenceOf<AnonymousULNGUUPTNLModifyItemIEExtensions>);
    aper_sequence_of!(ULNGUUPTNLModifyItemIEExtensions, AnonymousULNGUUPTNLModifyItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "UL-NGU-UP-TNLModifyItem")]
    #[non_exhaustive]
    pub struct ULNGUUPTNLModifyItem {
        #[rasn(identifier = "uL-NGU-UP-TNLInformation")]
        pub u_l_ngu_up_tnlinformation: UPTransportLayerInformation,
        #[rasn(identifier = "dL-NGU-UP-TNLInformation")]
        pub d_l_ngu_up_tnlinformation: UPTransportLayerInformation,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<ULNGUUPTNLModifyItemIEExtensions>,
    }
    impl ULNGUUPTNLModifyItem {
        pub fn new(
            u_l_ngu_up_tnlinformation: UPTransportLayerInformation,
            d_l_ngu_up_tnlinformation: UPTransportLayerInformation,
            i_e_extensions: Option<ULNGUUPTNLModifyItemIEExtensions>,
        ) -> Self {
            Self { u_l_ngu_up_tnlinformation, d_l_ngu_up_tnlinformation, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=4"), identifier = "UL-NGU-UP-TNLModifyList")]
    pub struct ULNGUUPTNLModifyList(pub SequenceOf<ULNGUUPTNLModifyItem>);
    aper_sequence_of!(ULNGUUPTNLModifyList, ULNGUUPTNLModifyItem, 1, 4);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    #[non_exhaustive]
    pub enum ULForwarding {
        #[rasn(identifier = "ul-forwarding-proposed")]
        ul_forwarding_proposed = 0,
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct UPTransportLayerInformationChoiceExtensions {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl UPTransportLayerInformationChoiceExtensions {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum UPTransportLayerInformation {
        gTPTunnel(GTPTunnel),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(UPTransportLayerInformationChoiceExtensions),
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUPTransportLayerInformationItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousUPTransportLayerInformationItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct UPTransportLayerInformationItemIEExtensions(
        pub SequenceOf<AnonymousUPTransportLayerInformationItemIEExtensions>,
    );
    aper_sequence_of!(
        UPTransportLayerInformationItemIEExtensions,
        AnonymousUPTransportLayerInformationItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UPTransportLayerInformationItem {
        #[rasn(identifier = "nGU-UP-TNLInformation")]
        pub n_gu_up_tnlinformation: UPTransportLayerInformation,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<UPTransportLayerInformationItemIEExtensions>,
    }
    impl UPTransportLayerInformationItem {
        pub fn new(
            n_gu_up_tnlinformation: UPTransportLayerInformation,
            i_e_extensions: Option<UPTransportLayerInformationItemIEExtensions>,
        ) -> Self {
            Self { n_gu_up_tnlinformation, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=3"))]
    pub struct UPTransportLayerInformationList(pub SequenceOf<UPTransportLayerInformationItem>);
    aper_sequence_of!(UPTransportLayerInformationList, UPTransportLayerInformationItem, 1, 3);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUPTransportLayerInformationPairItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousUPTransportLayerInformationPairItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct UPTransportLayerInformationPairItemIEExtensions(
        pub SequenceOf<AnonymousUPTransportLayerInformationPairItemIEExtensions>,
    );
    aper_sequence_of!(
        UPTransportLayerInformationPairItemIEExtensions,
        AnonymousUPTransportLayerInformationPairItemIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UPTransportLayerInformationPairItem {
        #[rasn(identifier = "uL-NGU-UP-TNLInformation")]
        pub u_l_ngu_up_tnlinformation: UPTransportLayerInformation,
        #[rasn(identifier = "dL-NGU-UP-TNLInformation")]
        pub d_l_ngu_up_tnlinformation: UPTransportLayerInformation,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<UPTransportLayerInformationPairItemIEExtensions>,
    }
    impl UPTransportLayerInformationPairItem {
        pub fn new(
            u_l_ngu_up_tnlinformation: UPTransportLayerInformation,
            d_l_ngu_up_tnlinformation: UPTransportLayerInformation,
            i_e_extensions: Option<UPTransportLayerInformationPairItemIEExtensions>,
        ) -> Self {
            Self { u_l_ngu_up_tnlinformation, d_l_ngu_up_tnlinformation, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=3"))]
    pub struct UPTransportLayerInformationPairList(pub SequenceOf<UPTransportLayerInformationPairItem>);
    aper_sequence_of!(UPTransportLayerInformationPairList, UPTransportLayerInformationPairItem, 1, 3);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUnavailableGUAMIItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousUnavailableGUAMIItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct UnavailableGUAMIItemIEExtensions(pub SequenceOf<AnonymousUnavailableGUAMIItemIEExtensions>);
    aper_sequence_of!(UnavailableGUAMIItemIEExtensions, AnonymousUnavailableGUAMIItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UnavailableGUAMIItem {
        #[rasn(identifier = "gUAMI")]
        pub g_uami: GUAMI,
        #[rasn(identifier = "timerApproachForGUAMIRemoval")]
        pub timer_approach_for_guamiremoval: Option<TimerApproachForGUAMIRemoval>,
        #[rasn(identifier = "backupAMFName")]
        pub backup_amfname: Option<AMFName>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<UnavailableGUAMIItemIEExtensions>,
    }
    impl UnavailableGUAMIItem {
        pub fn new(
            g_uami: GUAMI,
            timer_approach_for_guamiremoval: Option<TimerApproachForGUAMIRemoval>,
            backup_amfname: Option<AMFName>,
            i_e_extensions: Option<UnavailableGUAMIItemIEExtensions>,
        ) -> Self {
            Self { g_uami, timer_approach_for_guamiremoval, backup_amfname, i_e_extensions }
        }
    }
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=256"))]
    pub struct UnavailableGUAMIList(pub SequenceOf<UnavailableGUAMIItem>);
    aper_sequence_of!(UnavailableGUAMIList, UnavailableGUAMIItem, 1, 256);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct UserLocationInformationChoiceExtensions {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl UserLocationInformationChoiceExtensions {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum UserLocationInformation {
        userLocationInformationEUTRA(UserLocationInformationEUTRA),
        userLocationInformationNR(UserLocationInformationNR),
        userLocationInformationN3IWF(UserLocationInformationN3IWF),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(UserLocationInformationChoiceExtensions),
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUserLocationInformationEUTRAIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousUserLocationInformationEUTRAIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct UserLocationInformationEUTRAIEExtensions(
        pub SequenceOf<AnonymousUserLocationInformationEUTRAIEExtensions>,
    );
    aper_sequence_of!(
        UserLocationInformationEUTRAIEExtensions,
        AnonymousUserLocationInformationEUTRAIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UserLocationInformationEUTRA {
        #[rasn(identifier = "eUTRA-CGI")]
        pub e_utra_cgi: EUTRACGI,
        #[rasn(identifier = "tAI")]
        pub t_ai: TAI,
        #[rasn(identifier = "timeStamp")]
        pub time_stamp: Option<TimeStamp>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<UserLocationInformationEUTRAIEExtensions>,
    }
    impl UserLocationInformationEUTRA {
        pub fn new(
            e_utra_cgi: EUTRACGI,
            t_ai: TAI,
            time_stamp: Option<TimeStamp>,
            i_e_extensions: Option<UserLocationInformationEUTRAIEExtensions>,
        ) -> Self {
            Self { e_utra_cgi, t_ai, time_stamp, i_e_extensions }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUserLocationInformationN3IWFIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousUserLocationInformationN3IWFIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct UserLocationInformationN3IWFIEExtensions(
        pub SequenceOf<AnonymousUserLocationInformationN3IWFIEExtensions>,
    );
    aper_sequence_of!(
        UserLocationInformationN3IWFIEExtensions,
        AnonymousUserLocationInformationN3IWFIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UserLocationInformationN3IWF {
        #[rasn(identifier = "iPAddress")]
        pub i_paddress: TransportLayerAddress,
        #[rasn(identifier = "portNumber")]
        pub port_number: PortNumber,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<UserLocationInformationN3IWFIEExtensions>,
    }
    impl UserLocationInformationN3IWF {
        pub fn new(
            i_paddress: TransportLayerAddress,
            port_number: PortNumber,
            i_e_extensions: Option<UserLocationInformationN3IWFIEExtensions>,
        ) -> Self {
            Self { i_paddress, port_number, i_e_extensions }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUserLocationInformationNRIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousUserLocationInformationNRIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct UserLocationInformationNRIEExtensions(pub SequenceOf<AnonymousUserLocationInformationNRIEExtensions>);
    aper_sequence_of!(UserLocationInformationNRIEExtensions, AnonymousUserLocationInformationNRIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UserLocationInformationNR {
        #[rasn(identifier = "nR-CGI")]
        pub n_r_cgi: NRCGI,
        #[rasn(identifier = "tAI")]
        pub t_ai: TAI,
        #[rasn(identifier = "timeStamp")]
        pub time_stamp: Option<TimeStamp>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<UserLocationInformationNRIEExtensions>,
    }
    impl UserLocationInformationNR {
        pub fn new(
            n_r_cgi: NRCGI,
            t_ai: TAI,
            time_stamp: Option<TimeStamp>,
            i_e_extensions: Option<UserLocationInformationNRIEExtensions>,
        ) -> Self {
            Self { n_r_cgi, t_ai, time_stamp, i_e_extensions }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUserPlaneSecurityInformationIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousUserPlaneSecurityInformationIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct UserPlaneSecurityInformationIEExtensions(
        pub SequenceOf<AnonymousUserPlaneSecurityInformationIEExtensions>,
    );
    aper_sequence_of!(
        UserPlaneSecurityInformationIEExtensions,
        AnonymousUserPlaneSecurityInformationIEExtensions,
        1,
        65535
    );
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UserPlaneSecurityInformation {
        #[rasn(identifier = "securityResult")]
        pub security_result: SecurityResult,
        #[rasn(identifier = "securityIndication")]
        pub security_indication: SecurityIndication,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<UserPlaneSecurityInformationIEExtensions>,
    }
    impl UserPlaneSecurityInformation {
        pub fn new(
            security_result: SecurityResult,
            security_indication: SecurityIndication,
            i_e_extensions: Option<UserPlaneSecurityInformationIEExtensions>,
        ) -> Self {
            Self { security_result, security_indication, i_e_extensions }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousVolumeTimedReportItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousVolumeTimedReportItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct VolumeTimedReportItemIEExtensions(pub SequenceOf<AnonymousVolumeTimedReportItemIEExtensions>);
    aper_sequence_of!(VolumeTimedReportItemIEExtensions, AnonymousVolumeTimedReportItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "VolumeTimedReport-Item")]
    #[non_exhaustive]
    pub struct VolumeTimedReportItem {
        #[rasn(size("4"), identifier = "startTimeStamp")]
        pub start_time_stamp: OctetString,
        #[rasn(size("4"), identifier = "endTimeStamp")]
        pub end_time_stamp: OctetString,
        #[rasn(value("0..=18446744073709551615"), identifier = "usageCountUL")]
        pub usage_count_ul: u64,
        #[rasn(value("0..=18446744073709551615"), identifier = "usageCountDL")]
        pub usage_count_dl: u64,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<VolumeTimedReportItemIEExtensions>,
    }
    impl VolumeTimedReportItem {
        pub fn new(
            start_time_stamp: OctetString,
            end_time_stamp: OctetString,
            usage_count_ul: u64,
            usage_count_dl: u64,
            i_e_extensions: Option<VolumeTimedReportItemIEExtensions>,
        ) -> Self {
            Self { start_time_stamp, end_time_stamp, usage_count_ul, usage_count_dl, i_e_extensions }
        }
    }
    #[doc = " V"]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=2"))]
    pub struct VolumeTimedReportList(pub SequenceOf<VolumeTimedReportItem>);
    aper_sequence_of!(VolumeTimedReportList, VolumeTimedReportItem, 1, 2);
    #[doc = " W"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=1024"))]
    pub struct WarningAreaCoordinates(pub OctetString);
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct WarningAreaListChoiceExtensions {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl WarningAreaListChoiceExtensions {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags)]
    pub enum WarningAreaList {
        #[rasn(identifier = "eUTRA-CGIListForWarning")]
        eUTRA_CGIListForWarning(EUTRACGIListForWarning),
        #[rasn(identifier = "nR-CGIListForWarning")]
        nR_CGIListForWarning(NRCGIListForWarning),
        tAIListForWarning(TAIListForWarning),
        emergencyAreaIDList(EmergencyAreaIDList),
        #[rasn(identifier = "choice-Extensions")]
        choice_Extensions(WarningAreaListChoiceExtensions),
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=9600"))]
    pub struct WarningMessageContents(pub OctetString);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct WarningSecurityInfo(pub FixedOctetString<50usize>);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(delegate)]
    pub struct WarningType(pub FixedOctetString<2usize>);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousXnExtTLAItemIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousXnExtTLAItemIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct XnExtTLAItemIEExtensions(pub SequenceOf<AnonymousXnExtTLAItemIEExtensions>);
    aper_sequence_of!(XnExtTLAItemIEExtensions, AnonymousXnExtTLAItemIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "XnExtTLA-Item")]
    #[non_exhaustive]
    pub struct XnExtTLAItem {
        #[rasn(identifier = "iPsecTLA")]
        pub i_psec_tla: Option<TransportLayerAddress>,
        #[rasn(identifier = "gTP-TLAs")]
        pub g_tp_tlas: Option<XnGTPTLAs>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<XnExtTLAItemIEExtensions>,
    }
    impl XnExtTLAItem {
        pub fn new(
            i_psec_tla: Option<TransportLayerAddress>,
            g_tp_tlas: Option<XnGTPTLAs>,
            i_e_extensions: Option<XnExtTLAItemIEExtensions>,
        ) -> Self {
            Self { i_psec_tla, g_tp_tlas, i_e_extensions }
        }
    }
    #[doc = " X"]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"))]
    pub struct XnExtTLAs(pub SequenceOf<XnExtTLAItem>);
    aper_sequence_of!(XnExtTLAs, XnExtTLAItem, 1, 16);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=16"), identifier = "XnGTP-TLAs")]
    pub struct XnGTPTLAs(pub SequenceOf<TransportLayerAddress>);
    aper_sequence_of!(XnGTPTLAs, TransportLayerAddress, 1, 16);
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=2"))]
    pub struct XnTLAs(pub SequenceOf<TransportLayerAddress>);
    aper_sequence_of!(XnTLAs, TransportLayerAddress, 1, 2);
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousXnTNLConfigurationInfoIEExtensions {
        pub id: ProtocolExtensionID,
        pub criticality: Criticality,
        #[rasn(identifier = "extensionValue")]
        pub extension_value: Any,
    }
    impl AnonymousXnTNLConfigurationInfoIEExtensions {
        pub fn new(id: ProtocolExtensionID, criticality: Criticality, extension_value: Any) -> Self {
            Self { id, criticality, extension_value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct XnTNLConfigurationInfoIEExtensions(pub SequenceOf<AnonymousXnTNLConfigurationInfoIEExtensions>);
    aper_sequence_of!(XnTNLConfigurationInfoIEExtensions, AnonymousXnTNLConfigurationInfoIEExtensions, 1, 65535);
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct XnTNLConfigurationInfo {
        #[rasn(identifier = "xnTransportLayerAddresses")]
        pub xn_transport_layer_addresses: XnTLAs,
        #[rasn(identifier = "xnExtendedTransportLayerAddresses")]
        pub xn_extended_transport_layer_addresses: Option<XnExtTLAs>,
        #[rasn(identifier = "iE-Extensions")]
        pub i_e_extensions: Option<XnTNLConfigurationInfoIEExtensions>,
    }
    impl XnTNLConfigurationInfo {
        pub fn new(
            xn_transport_layer_addresses: XnTLAs,
            xn_extended_transport_layer_addresses: Option<XnExtTLAs>,
            i_e_extensions: Option<XnTNLConfigurationInfoIEExtensions>,
        ) -> Self {
            Self { xn_transport_layer_addresses, xn_extended_transport_layer_addresses, i_e_extensions }
        }
    }
}
#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, unused, clippy::too_many_arguments)]
pub mod ngap_pdu_contents {
    extern crate alloc;
    use super::ngap_common_data_types::*;
    use super::ngap_constants::*;
    use super::ngap_containers::*;
    use super::ngap_ies::*;
    use core::borrow::Borrow;
    use rasn::prelude::*;
    use std::sync::LazyLock;
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAMFConfigurationUpdateProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAMFConfigurationUpdateProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAMFConfigurationUpdateProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousAMFConfigurationUpdateProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousAMFConfigurationUpdateProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct AMFConfigurationUpdateProtocolIEs(pub SequenceOf<AnonymousAMFConfigurationUpdateProtocolIEs>);
    aper_sequence_of!(AMFConfigurationUpdateProtocolIEs, AnonymousAMFConfigurationUpdateProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " AMF Configuration Update Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " AMF CONFIGURATION UPDATE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AMFConfigurationUpdate {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: AMFConfigurationUpdateProtocolIEs,
    }
    impl AMFConfigurationUpdate {
        pub fn new(protocol_ies: AMFConfigurationUpdateProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAMFConfigurationUpdateAcknowledgeProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAMFConfigurationUpdateAcknowledgeProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAMFConfigurationUpdateAcknowledgeProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousAMFConfigurationUpdateAcknowledgeProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousAMFConfigurationUpdateAcknowledgeProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct AMFConfigurationUpdateAcknowledgeProtocolIEs(
        pub SequenceOf<AnonymousAMFConfigurationUpdateAcknowledgeProtocolIEs>,
    );
    aper_sequence_of!(
        AMFConfigurationUpdateAcknowledgeProtocolIEs,
        AnonymousAMFConfigurationUpdateAcknowledgeProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " AMF CONFIGURATION UPDATE ACKNOWLEDGE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AMFConfigurationUpdateAcknowledge {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: AMFConfigurationUpdateAcknowledgeProtocolIEs,
    }
    impl AMFConfigurationUpdateAcknowledge {
        pub fn new(protocol_ies: AMFConfigurationUpdateAcknowledgeProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAMFConfigurationUpdateFailureProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAMFConfigurationUpdateFailureProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAMFConfigurationUpdateFailureProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousAMFConfigurationUpdateFailureProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousAMFConfigurationUpdateFailureProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct AMFConfigurationUpdateFailureProtocolIEs(
        pub SequenceOf<AnonymousAMFConfigurationUpdateFailureProtocolIEs>,
    );
    aper_sequence_of!(
        AMFConfigurationUpdateFailureProtocolIEs,
        AnonymousAMFConfigurationUpdateFailureProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " AMF CONFIGURATION UPDATE FAILURE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AMFConfigurationUpdateFailure {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: AMFConfigurationUpdateFailureProtocolIEs,
    }
    impl AMFConfigurationUpdateFailure {
        pub fn new(protocol_ies: AMFConfigurationUpdateFailureProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousAMFStatusIndicationProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousAMFStatusIndicationProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousAMFStatusIndicationProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousAMFStatusIndicationProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousAMFStatusIndicationProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct AMFStatusIndicationProtocolIEs(pub SequenceOf<AnonymousAMFStatusIndicationProtocolIEs>);
    aper_sequence_of!(AMFStatusIndicationProtocolIEs, AnonymousAMFStatusIndicationProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " AMF Status Indication Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " AMF STATUS INDICATION"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct AMFStatusIndication {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: AMFStatusIndicationProtocolIEs,
    }
    impl AMFStatusIndication {
        pub fn new(protocol_ies: AMFStatusIndicationProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousCellTrafficTraceProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousCellTrafficTraceProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousCellTrafficTraceProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousCellTrafficTraceProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousCellTrafficTraceProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct CellTrafficTraceProtocolIEs(pub SequenceOf<AnonymousCellTrafficTraceProtocolIEs>);
    aper_sequence_of!(CellTrafficTraceProtocolIEs, AnonymousCellTrafficTraceProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " CELL TRAFFIC TRACE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct CellTrafficTrace {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: CellTrafficTraceProtocolIEs,
    }
    impl CellTrafficTrace {
        pub fn new(protocol_ies: CellTrafficTraceProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDeactivateTraceProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDeactivateTraceProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDeactivateTraceProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousDeactivateTraceProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousDeactivateTraceProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct DeactivateTraceProtocolIEs(pub SequenceOf<AnonymousDeactivateTraceProtocolIEs>);
    aper_sequence_of!(DeactivateTraceProtocolIEs, AnonymousDeactivateTraceProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " DEACTIVATE TRACE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct DeactivateTrace {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: DeactivateTraceProtocolIEs,
    }
    impl DeactivateTrace {
        pub fn new(protocol_ies: DeactivateTraceProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDownlinkNASTransportProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDownlinkNASTransportProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDownlinkNASTransportProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousDownlinkNASTransportProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousDownlinkNASTransportProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct DownlinkNASTransportProtocolIEs(pub SequenceOf<AnonymousDownlinkNASTransportProtocolIEs>);
    aper_sequence_of!(DownlinkNASTransportProtocolIEs, AnonymousDownlinkNASTransportProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " DOWNLINK NAS TRANSPORT"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct DownlinkNASTransport {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: DownlinkNASTransportProtocolIEs,
    }
    impl DownlinkNASTransport {
        pub fn new(protocol_ies: DownlinkNASTransportProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDownlinkNonUEAssociatedNRPPaTransportProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDownlinkNonUEAssociatedNRPPaTransportProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDownlinkNonUEAssociatedNRPPaTransportProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousDownlinkNonUEAssociatedNRPPaTransportProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousDownlinkNonUEAssociatedNRPPaTransportProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct DownlinkNonUEAssociatedNRPPaTransportProtocolIEs(
        pub SequenceOf<AnonymousDownlinkNonUEAssociatedNRPPaTransportProtocolIEs>,
    );
    aper_sequence_of!(
        DownlinkNonUEAssociatedNRPPaTransportProtocolIEs,
        AnonymousDownlinkNonUEAssociatedNRPPaTransportProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " DOWNLINK NON UE ASSOCIATED NRPPA TRANSPORT"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct DownlinkNonUEAssociatedNRPPaTransport {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: DownlinkNonUEAssociatedNRPPaTransportProtocolIEs,
    }
    impl DownlinkNonUEAssociatedNRPPaTransport {
        pub fn new(protocol_ies: DownlinkNonUEAssociatedNRPPaTransportProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDownlinkRANConfigurationTransferProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDownlinkRANConfigurationTransferProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDownlinkRANConfigurationTransferProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousDownlinkRANConfigurationTransferProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousDownlinkRANConfigurationTransferProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct DownlinkRANConfigurationTransferProtocolIEs(
        pub SequenceOf<AnonymousDownlinkRANConfigurationTransferProtocolIEs>,
    );
    aper_sequence_of!(
        DownlinkRANConfigurationTransferProtocolIEs,
        AnonymousDownlinkRANConfigurationTransferProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " DOWNLINK RAN CONFIGURATION TRANSFER"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct DownlinkRANConfigurationTransfer {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: DownlinkRANConfigurationTransferProtocolIEs,
    }
    impl DownlinkRANConfigurationTransfer {
        pub fn new(protocol_ies: DownlinkRANConfigurationTransferProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDownlinkRANStatusTransferProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDownlinkRANStatusTransferProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDownlinkRANStatusTransferProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousDownlinkRANStatusTransferProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousDownlinkRANStatusTransferProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct DownlinkRANStatusTransferProtocolIEs(pub SequenceOf<AnonymousDownlinkRANStatusTransferProtocolIEs>);
    aper_sequence_of!(DownlinkRANStatusTransferProtocolIEs, AnonymousDownlinkRANStatusTransferProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Downlink RAN Status Transfer Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " DOWNLINK RAN STATUS TRANSFER"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct DownlinkRANStatusTransfer {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: DownlinkRANStatusTransferProtocolIEs,
    }
    impl DownlinkRANStatusTransfer {
        pub fn new(protocol_ies: DownlinkRANStatusTransferProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousDownlinkUEAssociatedNRPPaTransportProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousDownlinkUEAssociatedNRPPaTransportProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousDownlinkUEAssociatedNRPPaTransportProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousDownlinkUEAssociatedNRPPaTransportProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousDownlinkUEAssociatedNRPPaTransportProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct DownlinkUEAssociatedNRPPaTransportProtocolIEs(
        pub SequenceOf<AnonymousDownlinkUEAssociatedNRPPaTransportProtocolIEs>,
    );
    aper_sequence_of!(
        DownlinkUEAssociatedNRPPaTransportProtocolIEs,
        AnonymousDownlinkUEAssociatedNRPPaTransportProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " NRPPA TRANSPORT ELEMENTARY PROCEDURES"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " DOWNLINK UE ASSOCIATED NRPPA TRANSPORT"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct DownlinkUEAssociatedNRPPaTransport {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: DownlinkUEAssociatedNRPPaTransportProtocolIEs,
    }
    impl DownlinkUEAssociatedNRPPaTransport {
        pub fn new(protocol_ies: DownlinkUEAssociatedNRPPaTransportProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousErrorIndicationProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousErrorIndicationProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousErrorIndicationProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousErrorIndicationProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousErrorIndicationProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct ErrorIndicationProtocolIEs(pub SequenceOf<AnonymousErrorIndicationProtocolIEs>);
    aper_sequence_of!(ErrorIndicationProtocolIEs, AnonymousErrorIndicationProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Error Indication Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " ERROR INDICATION"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct ErrorIndication {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: ErrorIndicationProtocolIEs,
    }
    impl ErrorIndication {
        pub fn new(protocol_ies: ErrorIndicationProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverCancelProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverCancelProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverCancelProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousHandoverCancelProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousHandoverCancelProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct HandoverCancelProtocolIEs(pub SequenceOf<AnonymousHandoverCancelProtocolIEs>);
    aper_sequence_of!(HandoverCancelProtocolIEs, AnonymousHandoverCancelProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Handover Cancellation Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " HANDOVER CANCEL"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverCancel {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: HandoverCancelProtocolIEs,
    }
    impl HandoverCancel {
        pub fn new(protocol_ies: HandoverCancelProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverCancelAcknowledgeProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverCancelAcknowledgeProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverCancelAcknowledgeProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousHandoverCancelAcknowledgeProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousHandoverCancelAcknowledgeProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct HandoverCancelAcknowledgeProtocolIEs(pub SequenceOf<AnonymousHandoverCancelAcknowledgeProtocolIEs>);
    aper_sequence_of!(HandoverCancelAcknowledgeProtocolIEs, AnonymousHandoverCancelAcknowledgeProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " HANDOVER CANCEL ACKNOWLEDGE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverCancelAcknowledge {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: HandoverCancelAcknowledgeProtocolIEs,
    }
    impl HandoverCancelAcknowledge {
        pub fn new(protocol_ies: HandoverCancelAcknowledgeProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverCommandProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverCommandProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverCommandProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousHandoverCommandProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousHandoverCommandProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct HandoverCommandProtocolIEs(pub SequenceOf<AnonymousHandoverCommandProtocolIEs>);
    aper_sequence_of!(HandoverCommandProtocolIEs, AnonymousHandoverCommandProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " HANDOVER COMMAND"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverCommand {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: HandoverCommandProtocolIEs,
    }
    impl HandoverCommand {
        pub fn new(protocol_ies: HandoverCommandProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverFailureProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverFailureProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverFailureProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousHandoverFailureProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousHandoverFailureProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct HandoverFailureProtocolIEs(pub SequenceOf<AnonymousHandoverFailureProtocolIEs>);
    aper_sequence_of!(HandoverFailureProtocolIEs, AnonymousHandoverFailureProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " HANDOVER FAILURE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverFailure {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: HandoverFailureProtocolIEs,
    }
    impl HandoverFailure {
        pub fn new(protocol_ies: HandoverFailureProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverNotifyProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverNotifyProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverNotifyProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousHandoverNotifyProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousHandoverNotifyProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct HandoverNotifyProtocolIEs(pub SequenceOf<AnonymousHandoverNotifyProtocolIEs>);
    aper_sequence_of!(HandoverNotifyProtocolIEs, AnonymousHandoverNotifyProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Handover Notification Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " HANDOVER NOTIFY"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverNotify {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: HandoverNotifyProtocolIEs,
    }
    impl HandoverNotify {
        pub fn new(protocol_ies: HandoverNotifyProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverPreparationFailureProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverPreparationFailureProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverPreparationFailureProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousHandoverPreparationFailureProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousHandoverPreparationFailureProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct HandoverPreparationFailureProtocolIEs(pub SequenceOf<AnonymousHandoverPreparationFailureProtocolIEs>);
    aper_sequence_of!(HandoverPreparationFailureProtocolIEs, AnonymousHandoverPreparationFailureProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " HANDOVER PREPARATION FAILURE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverPreparationFailure {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: HandoverPreparationFailureProtocolIEs,
    }
    impl HandoverPreparationFailure {
        pub fn new(protocol_ies: HandoverPreparationFailureProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverRequestProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverRequestProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverRequestProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousHandoverRequestProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousHandoverRequestProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct HandoverRequestProtocolIEs(pub SequenceOf<AnonymousHandoverRequestProtocolIEs>);
    aper_sequence_of!(HandoverRequestProtocolIEs, AnonymousHandoverRequestProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Handover Resource Allocation Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " HANDOVER REQUEST"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverRequest {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: HandoverRequestProtocolIEs,
    }
    impl HandoverRequest {
        pub fn new(protocol_ies: HandoverRequestProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverRequestAcknowledgeProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverRequestAcknowledgeProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverRequestAcknowledgeProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousHandoverRequestAcknowledgeProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousHandoverRequestAcknowledgeProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct HandoverRequestAcknowledgeProtocolIEs(pub SequenceOf<AnonymousHandoverRequestAcknowledgeProtocolIEs>);
    aper_sequence_of!(HandoverRequestAcknowledgeProtocolIEs, AnonymousHandoverRequestAcknowledgeProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " HANDOVER REQUEST ACKNOWLEDGE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverRequestAcknowledge {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: HandoverRequestAcknowledgeProtocolIEs,
    }
    impl HandoverRequestAcknowledge {
        pub fn new(protocol_ies: HandoverRequestAcknowledgeProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousHandoverRequiredProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousHandoverRequiredProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousHandoverRequiredProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousHandoverRequiredProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousHandoverRequiredProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct HandoverRequiredProtocolIEs(pub SequenceOf<AnonymousHandoverRequiredProtocolIEs>);
    aper_sequence_of!(HandoverRequiredProtocolIEs, AnonymousHandoverRequiredProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE MOBILITY MANAGEMENT ELEMENTARY PROCEDURES"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Handover Preparation Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " HANDOVER REQUIRED"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct HandoverRequired {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: HandoverRequiredProtocolIEs,
    }
    impl HandoverRequired {
        pub fn new(protocol_ies: HandoverRequiredProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousInitialContextSetupFailureProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousInitialContextSetupFailureProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousInitialContextSetupFailureProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousInitialContextSetupFailureProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousInitialContextSetupFailureProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct InitialContextSetupFailureProtocolIEs(pub SequenceOf<AnonymousInitialContextSetupFailureProtocolIEs>);
    aper_sequence_of!(InitialContextSetupFailureProtocolIEs, AnonymousInitialContextSetupFailureProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " INITIAL CONTEXT SETUP FAILURE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct InitialContextSetupFailure {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: InitialContextSetupFailureProtocolIEs,
    }
    impl InitialContextSetupFailure {
        pub fn new(protocol_ies: InitialContextSetupFailureProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousInitialContextSetupRequestProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousInitialContextSetupRequestProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousInitialContextSetupRequestProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousInitialContextSetupRequestProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousInitialContextSetupRequestProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct InitialContextSetupRequestProtocolIEs(pub SequenceOf<AnonymousInitialContextSetupRequestProtocolIEs>);
    aper_sequence_of!(InitialContextSetupRequestProtocolIEs, AnonymousInitialContextSetupRequestProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE CONTEXT MANAGEMENT ELEMENTARY PROCEDURES"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Initial Context Setup Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " INITIAL CONTEXT SETUP REQUEST"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct InitialContextSetupRequest {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: InitialContextSetupRequestProtocolIEs,
    }
    impl InitialContextSetupRequest {
        pub fn new(protocol_ies: InitialContextSetupRequestProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousInitialContextSetupResponseProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousInitialContextSetupResponseProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousInitialContextSetupResponseProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousInitialContextSetupResponseProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousInitialContextSetupResponseProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct InitialContextSetupResponseProtocolIEs(pub SequenceOf<AnonymousInitialContextSetupResponseProtocolIEs>);
    aper_sequence_of!(
        InitialContextSetupResponseProtocolIEs,
        AnonymousInitialContextSetupResponseProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " INITIAL CONTEXT SETUP RESPONSE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct InitialContextSetupResponse {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: InitialContextSetupResponseProtocolIEs,
    }
    impl InitialContextSetupResponse {
        pub fn new(protocol_ies: InitialContextSetupResponseProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousInitialUEMessageProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousInitialUEMessageProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousInitialUEMessageProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousInitialUEMessageProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousInitialUEMessageProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct InitialUEMessageProtocolIEs(pub SequenceOf<AnonymousInitialUEMessageProtocolIEs>);
    aper_sequence_of!(InitialUEMessageProtocolIEs, AnonymousInitialUEMessageProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " NAS TRANSPORT ELEMENTARY PROCEDURES"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " INITIAL UE MESSAGE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct InitialUEMessage {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: InitialUEMessageProtocolIEs,
    }
    impl InitialUEMessage {
        pub fn new(protocol_ies: InitialUEMessageProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousLocationReportProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousLocationReportProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousLocationReportProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousLocationReportProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousLocationReportProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct LocationReportProtocolIEs(pub SequenceOf<AnonymousLocationReportProtocolIEs>);
    aper_sequence_of!(LocationReportProtocolIEs, AnonymousLocationReportProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " LOCATION REPORT"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct LocationReport {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: LocationReportProtocolIEs,
    }
    impl LocationReport {
        pub fn new(protocol_ies: LocationReportProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousLocationReportingControlProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousLocationReportingControlProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousLocationReportingControlProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousLocationReportingControlProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousLocationReportingControlProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct LocationReportingControlProtocolIEs(pub SequenceOf<AnonymousLocationReportingControlProtocolIEs>);
    aper_sequence_of!(LocationReportingControlProtocolIEs, AnonymousLocationReportingControlProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " LOCATION REPORTING ELEMENTARY PROCEDURES"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " LOCATION REPORTING CONTROL"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct LocationReportingControl {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: LocationReportingControlProtocolIEs,
    }
    impl LocationReportingControl {
        pub fn new(protocol_ies: LocationReportingControlProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousLocationReportingFailureIndicationProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousLocationReportingFailureIndicationProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousLocationReportingFailureIndicationProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousLocationReportingFailureIndicationProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousLocationReportingFailureIndicationProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct LocationReportingFailureIndicationProtocolIEs(
        pub SequenceOf<AnonymousLocationReportingFailureIndicationProtocolIEs>,
    );
    aper_sequence_of!(
        LocationReportingFailureIndicationProtocolIEs,
        AnonymousLocationReportingFailureIndicationProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " LOCATION REPORTING FAILURE INDICATION"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct LocationReportingFailureIndication {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: LocationReportingFailureIndicationProtocolIEs,
    }
    impl LocationReportingFailureIndication {
        pub fn new(protocol_ies: LocationReportingFailureIndicationProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousNASNonDeliveryIndicationProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousNASNonDeliveryIndicationProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousNASNonDeliveryIndicationProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousNASNonDeliveryIndicationProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousNASNonDeliveryIndicationProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct NASNonDeliveryIndicationProtocolIEs(pub SequenceOf<AnonymousNASNonDeliveryIndicationProtocolIEs>);
    aper_sequence_of!(NASNonDeliveryIndicationProtocolIEs, AnonymousNASNonDeliveryIndicationProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " NAS NON DELIVERY INDICATION"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct NASNonDeliveryIndication {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: NASNonDeliveryIndicationProtocolIEs,
    }
    impl NASNonDeliveryIndication {
        pub fn new(protocol_ies: NASNonDeliveryIndicationProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousNGResetProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousNGResetProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousNGResetProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousNGResetProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousNGResetProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct NGResetProtocolIEs(pub SequenceOf<AnonymousNGResetProtocolIEs>);
    aper_sequence_of!(NGResetProtocolIEs, AnonymousNGResetProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " NG Reset Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " NG RESET"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct NGReset {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: NGResetProtocolIEs,
    }
    impl NGReset {
        pub fn new(protocol_ies: NGResetProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousNGResetAcknowledgeProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousNGResetAcknowledgeProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousNGResetAcknowledgeProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousNGResetAcknowledgeProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousNGResetAcknowledgeProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct NGResetAcknowledgeProtocolIEs(pub SequenceOf<AnonymousNGResetAcknowledgeProtocolIEs>);
    aper_sequence_of!(NGResetAcknowledgeProtocolIEs, AnonymousNGResetAcknowledgeProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " NG RESET ACKNOWLEDGE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct NGResetAcknowledge {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: NGResetAcknowledgeProtocolIEs,
    }
    impl NGResetAcknowledge {
        pub fn new(protocol_ies: NGResetAcknowledgeProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousNGSetupFailureProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousNGSetupFailureProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousNGSetupFailureProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousNGSetupFailureProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousNGSetupFailureProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct NGSetupFailureProtocolIEs(pub SequenceOf<AnonymousNGSetupFailureProtocolIEs>);
    aper_sequence_of!(NGSetupFailureProtocolIEs, AnonymousNGSetupFailureProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " NG SETUP FAILURE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct NGSetupFailure {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: NGSetupFailureProtocolIEs,
    }
    impl NGSetupFailure {
        pub fn new(protocol_ies: NGSetupFailureProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousNGSetupRequestProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousNGSetupRequestProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousNGSetupRequestProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousNGSetupRequestProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousNGSetupRequestProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct NGSetupRequestProtocolIEs(pub SequenceOf<AnonymousNGSetupRequestProtocolIEs>);
    aper_sequence_of!(NGSetupRequestProtocolIEs, AnonymousNGSetupRequestProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " INTERFACE MANAGEMENT ELEMENTARY PROCEDURES"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " NG Setup Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " NG SETUP REQUEST"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct NGSetupRequest {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: NGSetupRequestProtocolIEs,
    }
    impl NGSetupRequest {
        pub fn new(protocol_ies: NGSetupRequestProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousNGSetupResponseProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousNGSetupResponseProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousNGSetupResponseProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousNGSetupResponseProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousNGSetupResponseProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct NGSetupResponseProtocolIEs(pub SequenceOf<AnonymousNGSetupResponseProtocolIEs>);
    aper_sequence_of!(NGSetupResponseProtocolIEs, AnonymousNGSetupResponseProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " NG SETUP RESPONSE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct NGSetupResponse {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: NGSetupResponseProtocolIEs,
    }
    impl NGSetupResponse {
        pub fn new(protocol_ies: NGSetupResponseProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousOverloadStartProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousOverloadStartProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousOverloadStartProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousOverloadStartProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousOverloadStartProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct OverloadStartProtocolIEs(pub SequenceOf<AnonymousOverloadStartProtocolIEs>);
    aper_sequence_of!(OverloadStartProtocolIEs, AnonymousOverloadStartProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " OVERLOAD START"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct OverloadStart {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: OverloadStartProtocolIEs,
    }
    impl OverloadStart {
        pub fn new(protocol_ies: OverloadStartProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousOverloadStopProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousOverloadStopProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousOverloadStopProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousOverloadStopProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousOverloadStopProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct OverloadStopProtocolIEs(pub SequenceOf<AnonymousOverloadStopProtocolIEs>);
    aper_sequence_of!(OverloadStopProtocolIEs, AnonymousOverloadStopProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " OVERLOAD STOP"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct OverloadStop {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: OverloadStopProtocolIEs,
    }
    impl OverloadStop {
        pub fn new(protocol_ies: OverloadStopProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyConfirmProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyConfirmProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyConfirmProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPDUSessionResourceModifyConfirmProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyConfirmProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PDUSessionResourceModifyConfirmProtocolIEs(
        pub SequenceOf<AnonymousPDUSessionResourceModifyConfirmProtocolIEs>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyConfirmProtocolIEs,
        AnonymousPDUSessionResourceModifyConfirmProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU SESSION RESOURCE MODIFY CONFIRM"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyConfirm {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PDUSessionResourceModifyConfirmProtocolIEs,
    }
    impl PDUSessionResourceModifyConfirm {
        pub fn new(protocol_ies: PDUSessionResourceModifyConfirmProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyIndicationProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyIndicationProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyIndicationProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPDUSessionResourceModifyIndicationProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyIndicationProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PDUSessionResourceModifyIndicationProtocolIEs(
        pub SequenceOf<AnonymousPDUSessionResourceModifyIndicationProtocolIEs>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyIndicationProtocolIEs,
        AnonymousPDUSessionResourceModifyIndicationProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU Session Resource Modify Indication Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU SESSION RESOURCE MODIFY INDICATION"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyIndication {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PDUSessionResourceModifyIndicationProtocolIEs,
    }
    impl PDUSessionResourceModifyIndication {
        pub fn new(protocol_ies: PDUSessionResourceModifyIndicationProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyRequestProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyRequestProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyRequestProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPDUSessionResourceModifyRequestProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyRequestProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PDUSessionResourceModifyRequestProtocolIEs(
        pub SequenceOf<AnonymousPDUSessionResourceModifyRequestProtocolIEs>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyRequestProtocolIEs,
        AnonymousPDUSessionResourceModifyRequestProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU Session Resource Modify Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU SESSION RESOURCE MODIFY REQUEST"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyRequest {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PDUSessionResourceModifyRequestProtocolIEs,
    }
    impl PDUSessionResourceModifyRequest {
        pub fn new(protocol_ies: PDUSessionResourceModifyRequestProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceModifyResponseProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceModifyResponseProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceModifyResponseProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPDUSessionResourceModifyResponseProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceModifyResponseProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PDUSessionResourceModifyResponseProtocolIEs(
        pub SequenceOf<AnonymousPDUSessionResourceModifyResponseProtocolIEs>,
    );
    aper_sequence_of!(
        PDUSessionResourceModifyResponseProtocolIEs,
        AnonymousPDUSessionResourceModifyResponseProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU SESSION RESOURCE MODIFY RESPONSE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceModifyResponse {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PDUSessionResourceModifyResponseProtocolIEs,
    }
    impl PDUSessionResourceModifyResponse {
        pub fn new(protocol_ies: PDUSessionResourceModifyResponseProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceNotifyProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceNotifyProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceNotifyProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPDUSessionResourceNotifyProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousPDUSessionResourceNotifyProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PDUSessionResourceNotifyProtocolIEs(pub SequenceOf<AnonymousPDUSessionResourceNotifyProtocolIEs>);
    aper_sequence_of!(PDUSessionResourceNotifyProtocolIEs, AnonymousPDUSessionResourceNotifyProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU Session Resource Notify Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU SESSION RESOURCE NOTIFY"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceNotify {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PDUSessionResourceNotifyProtocolIEs,
    }
    impl PDUSessionResourceNotify {
        pub fn new(protocol_ies: PDUSessionResourceNotifyProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceReleaseCommandProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceReleaseCommandProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceReleaseCommandProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPDUSessionResourceReleaseCommandProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceReleaseCommandProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PDUSessionResourceReleaseCommandProtocolIEs(
        pub SequenceOf<AnonymousPDUSessionResourceReleaseCommandProtocolIEs>,
    );
    aper_sequence_of!(
        PDUSessionResourceReleaseCommandProtocolIEs,
        AnonymousPDUSessionResourceReleaseCommandProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU Session Resource Release Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU SESSION RESOURCE RELEASE COMMAND"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceReleaseCommand {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PDUSessionResourceReleaseCommandProtocolIEs,
    }
    impl PDUSessionResourceReleaseCommand {
        pub fn new(protocol_ies: PDUSessionResourceReleaseCommandProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceReleaseResponseProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceReleaseResponseProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceReleaseResponseProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPDUSessionResourceReleaseResponseProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceReleaseResponseProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PDUSessionResourceReleaseResponseProtocolIEs(
        pub SequenceOf<AnonymousPDUSessionResourceReleaseResponseProtocolIEs>,
    );
    aper_sequence_of!(
        PDUSessionResourceReleaseResponseProtocolIEs,
        AnonymousPDUSessionResourceReleaseResponseProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU SESSION RESOURCE RELEASE RESPONSE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceReleaseResponse {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PDUSessionResourceReleaseResponseProtocolIEs,
    }
    impl PDUSessionResourceReleaseResponse {
        pub fn new(protocol_ies: PDUSessionResourceReleaseResponseProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceSetupRequestProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceSetupRequestProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceSetupRequestProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPDUSessionResourceSetupRequestProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceSetupRequestProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PDUSessionResourceSetupRequestProtocolIEs(
        pub SequenceOf<AnonymousPDUSessionResourceSetupRequestProtocolIEs>,
    );
    aper_sequence_of!(
        PDUSessionResourceSetupRequestProtocolIEs,
        AnonymousPDUSessionResourceSetupRequestProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU SESSION MANAGEMENT ELEMENTARY PROCEDURES"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU Session Resource Setup Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU SESSION RESOURCE SETUP REQUEST"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceSetupRequest {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PDUSessionResourceSetupRequestProtocolIEs,
    }
    impl PDUSessionResourceSetupRequest {
        pub fn new(protocol_ies: PDUSessionResourceSetupRequestProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPDUSessionResourceSetupResponseProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPDUSessionResourceSetupResponseProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPDUSessionResourceSetupResponseProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPDUSessionResourceSetupResponseProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousPDUSessionResourceSetupResponseProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PDUSessionResourceSetupResponseProtocolIEs(
        pub SequenceOf<AnonymousPDUSessionResourceSetupResponseProtocolIEs>,
    );
    aper_sequence_of!(
        PDUSessionResourceSetupResponseProtocolIEs,
        AnonymousPDUSessionResourceSetupResponseProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PDU SESSION RESOURCE SETUP RESPONSE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PDUSessionResourceSetupResponse {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PDUSessionResourceSetupResponseProtocolIEs,
    }
    impl PDUSessionResourceSetupResponse {
        pub fn new(protocol_ies: PDUSessionResourceSetupResponseProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPWSCancelRequestProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPWSCancelRequestProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPWSCancelRequestProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPWSCancelRequestProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousPWSCancelRequestProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PWSCancelRequestProtocolIEs(pub SequenceOf<AnonymousPWSCancelRequestProtocolIEs>);
    aper_sequence_of!(PWSCancelRequestProtocolIEs, AnonymousPWSCancelRequestProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PWS Cancel Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PWS CANCEL REQUEST"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PWSCancelRequest {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PWSCancelRequestProtocolIEs,
    }
    impl PWSCancelRequest {
        pub fn new(protocol_ies: PWSCancelRequestProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPWSCancelResponseProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPWSCancelResponseProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPWSCancelResponseProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPWSCancelResponseProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousPWSCancelResponseProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PWSCancelResponseProtocolIEs(pub SequenceOf<AnonymousPWSCancelResponseProtocolIEs>);
    aper_sequence_of!(PWSCancelResponseProtocolIEs, AnonymousPWSCancelResponseProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PWS CANCEL RESPONSE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PWSCancelResponse {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PWSCancelResponseProtocolIEs,
    }
    impl PWSCancelResponse {
        pub fn new(protocol_ies: PWSCancelResponseProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPWSFailureIndicationProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPWSFailureIndicationProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPWSFailureIndicationProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPWSFailureIndicationProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousPWSFailureIndicationProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PWSFailureIndicationProtocolIEs(pub SequenceOf<AnonymousPWSFailureIndicationProtocolIEs>);
    aper_sequence_of!(PWSFailureIndicationProtocolIEs, AnonymousPWSFailureIndicationProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PWS Failure Indication Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PWS FAILURE INDICATION"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PWSFailureIndication {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PWSFailureIndicationProtocolIEs,
    }
    impl PWSFailureIndication {
        pub fn new(protocol_ies: PWSFailureIndicationProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPWSRestartIndicationProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPWSRestartIndicationProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPWSRestartIndicationProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPWSRestartIndicationProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousPWSRestartIndicationProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PWSRestartIndicationProtocolIEs(pub SequenceOf<AnonymousPWSRestartIndicationProtocolIEs>);
    aper_sequence_of!(PWSRestartIndicationProtocolIEs, AnonymousPWSRestartIndicationProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PWS Restart Indication Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PWS RESTART INDICATION"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PWSRestartIndication {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PWSRestartIndicationProtocolIEs,
    }
    impl PWSRestartIndication {
        pub fn new(protocol_ies: PWSRestartIndicationProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPagingProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPagingProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPagingProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPagingProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousPagingProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PagingProtocolIEs(pub SequenceOf<AnonymousPagingProtocolIEs>);
    aper_sequence_of!(PagingProtocolIEs, AnonymousPagingProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PAGING ELEMENTARY PROCEDURE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PAGING"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct Paging {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PagingProtocolIEs,
    }
    impl Paging {
        pub fn new(protocol_ies: PagingProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPathSwitchRequestProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPathSwitchRequestProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPathSwitchRequestProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPathSwitchRequestProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousPathSwitchRequestProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PathSwitchRequestProtocolIEs(pub SequenceOf<AnonymousPathSwitchRequestProtocolIEs>);
    aper_sequence_of!(PathSwitchRequestProtocolIEs, AnonymousPathSwitchRequestProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Path Switch Request Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PATH SWITCH REQUEST"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PathSwitchRequest {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PathSwitchRequestProtocolIEs,
    }
    impl PathSwitchRequest {
        pub fn new(protocol_ies: PathSwitchRequestProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPathSwitchRequestAcknowledgeProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPathSwitchRequestAcknowledgeProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPathSwitchRequestAcknowledgeProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPathSwitchRequestAcknowledgeProtocolIEs {
        pub fn new(
            id: u16,
            criticality: AnonymousPathSwitchRequestAcknowledgeProtocolIEsCriticality,
            value: Any,
        ) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PathSwitchRequestAcknowledgeProtocolIEs(
        pub SequenceOf<AnonymousPathSwitchRequestAcknowledgeProtocolIEs>,
    );
    aper_sequence_of!(
        PathSwitchRequestAcknowledgeProtocolIEs,
        AnonymousPathSwitchRequestAcknowledgeProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PATH SWITCH REQUEST ACKNOWLEDGE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PathSwitchRequestAcknowledge {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PathSwitchRequestAcknowledgeProtocolIEs,
    }
    impl PathSwitchRequestAcknowledge {
        pub fn new(protocol_ies: PathSwitchRequestAcknowledgeProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash, Copy)]
    #[rasn(enumerated)]
    pub enum AnonymousPathSwitchRequestFailureProtocolIEsCriticality {
        reject = 0,
        ignore = 1,
        notify = 2,
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPathSwitchRequestFailureProtocolIEs {
        #[rasn(value("0..=65535"))]
        pub id: u16,
        pub criticality: AnonymousPathSwitchRequestFailureProtocolIEsCriticality,
        pub value: Any,
    }
    impl AnonymousPathSwitchRequestFailureProtocolIEs {
        pub fn new(id: u16, criticality: AnonymousPathSwitchRequestFailureProtocolIEsCriticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct PathSwitchRequestFailureProtocolIEs(pub SequenceOf<AnonymousPathSwitchRequestFailureProtocolIEs>);
    aper_sequence_of!(PathSwitchRequestFailureProtocolIEs, AnonymousPathSwitchRequestFailureProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PATH SWITCH REQUEST FAILURE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PathSwitchRequestFailure {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: PathSwitchRequestFailureProtocolIEs,
    }
    impl PathSwitchRequestFailure {
        pub fn new(protocol_ies: PathSwitchRequestFailureProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousPrivateMessagePrivateIEs {
        pub id: PrivateIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousPrivateMessagePrivateIEs {
        pub fn new(id: PrivateIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("1..=65535"))]
    pub struct PrivateMessagePrivateIEs(pub SequenceOf<AnonymousPrivateMessagePrivateIEs>);
    aper_sequence_of!(PrivateMessagePrivateIEs, AnonymousPrivateMessagePrivateIEs, 1, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PRIVATE MESSAGE ELEMENTARY PROCEDURE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " PRIVATE MESSAGE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct PrivateMessage {
        #[rasn(identifier = "privateIEs")]
        pub private_ies: PrivateMessagePrivateIEs,
    }
    impl PrivateMessage {
        pub fn new(private_ies: PrivateMessagePrivateIEs) -> Self {
            Self { private_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousRANConfigurationUpdateProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousRANConfigurationUpdateProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct RANConfigurationUpdateProtocolIEs(pub SequenceOf<AnonymousRANConfigurationUpdateProtocolIEs>);
    aper_sequence_of!(RANConfigurationUpdateProtocolIEs, AnonymousRANConfigurationUpdateProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " RAN Configuration Update Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " RAN CONFIGURATION UPDATE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct RANConfigurationUpdate {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: RANConfigurationUpdateProtocolIEs,
    }
    impl RANConfigurationUpdate {
        pub fn new(protocol_ies: RANConfigurationUpdateProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousRANConfigurationUpdateAcknowledgeProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousRANConfigurationUpdateAcknowledgeProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct RANConfigurationUpdateAcknowledgeProtocolIEs(
        pub SequenceOf<AnonymousRANConfigurationUpdateAcknowledgeProtocolIEs>,
    );
    aper_sequence_of!(
        RANConfigurationUpdateAcknowledgeProtocolIEs,
        AnonymousRANConfigurationUpdateAcknowledgeProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " RAN CONFIGURATION UPDATE ACKNOWLEDGE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct RANConfigurationUpdateAcknowledge {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: RANConfigurationUpdateAcknowledgeProtocolIEs,
    }
    impl RANConfigurationUpdateAcknowledge {
        pub fn new(protocol_ies: RANConfigurationUpdateAcknowledgeProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousRANConfigurationUpdateFailureProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousRANConfigurationUpdateFailureProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct RANConfigurationUpdateFailureProtocolIEs(
        pub SequenceOf<AnonymousRANConfigurationUpdateFailureProtocolIEs>,
    );
    aper_sequence_of!(
        RANConfigurationUpdateFailureProtocolIEs,
        AnonymousRANConfigurationUpdateFailureProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " RAN CONFIGURATION UPDATE FAILURE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct RANConfigurationUpdateFailure {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: RANConfigurationUpdateFailureProtocolIEs,
    }
    impl RANConfigurationUpdateFailure {
        pub fn new(protocol_ies: RANConfigurationUpdateFailureProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousRRCInactiveTransitionReportProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousRRCInactiveTransitionReportProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct RRCInactiveTransitionReportProtocolIEs(pub SequenceOf<AnonymousRRCInactiveTransitionReportProtocolIEs>);
    aper_sequence_of!(
        RRCInactiveTransitionReportProtocolIEs,
        AnonymousRRCInactiveTransitionReportProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " RRC INACTIVE TRANSITION REPORT"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct RRCInactiveTransitionReport {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: RRCInactiveTransitionReportProtocolIEs,
    }
    impl RRCInactiveTransitionReport {
        pub fn new(protocol_ies: RRCInactiveTransitionReportProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousRerouteNASRequestProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousRerouteNASRequestProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct RerouteNASRequestProtocolIEs(pub SequenceOf<AnonymousRerouteNASRequestProtocolIEs>);
    aper_sequence_of!(RerouteNASRequestProtocolIEs, AnonymousRerouteNASRequestProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " REROUTE NAS REQUEST"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct RerouteNASRequest {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: RerouteNASRequestProtocolIEs,
    }
    impl RerouteNASRequest {
        pub fn new(protocol_ies: RerouteNASRequestProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousSecondaryRATDataUsageReportProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousSecondaryRATDataUsageReportProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct SecondaryRATDataUsageReportProtocolIEs(pub SequenceOf<AnonymousSecondaryRATDataUsageReportProtocolIEs>);
    aper_sequence_of!(
        SecondaryRATDataUsageReportProtocolIEs,
        AnonymousSecondaryRATDataUsageReportProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " DATA USAGE REPORTING ELEMENTARY PROCEDURES"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " SECONDARY RAT DATA USAGE REPORT"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct SecondaryRATDataUsageReport {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: SecondaryRATDataUsageReportProtocolIEs,
    }
    impl SecondaryRATDataUsageReport {
        pub fn new(protocol_ies: SecondaryRATDataUsageReportProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTraceFailureIndicationProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousTraceFailureIndicationProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct TraceFailureIndicationProtocolIEs(pub SequenceOf<AnonymousTraceFailureIndicationProtocolIEs>);
    aper_sequence_of!(TraceFailureIndicationProtocolIEs, AnonymousTraceFailureIndicationProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " TRACE FAILURE INDICATION"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct TraceFailureIndication {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: TraceFailureIndicationProtocolIEs,
    }
    impl TraceFailureIndication {
        pub fn new(protocol_ies: TraceFailureIndicationProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousTraceStartProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousTraceStartProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct TraceStartProtocolIEs(pub SequenceOf<AnonymousTraceStartProtocolIEs>);
    aper_sequence_of!(TraceStartProtocolIEs, AnonymousTraceStartProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " TRACE ELEMENTARY PROCEDURES"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " TRACE START"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct TraceStart {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: TraceStartProtocolIEs,
    }
    impl TraceStart {
        pub fn new(protocol_ies: TraceStartProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUEContextModificationFailureProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUEContextModificationFailureProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UEContextModificationFailureProtocolIEs(
        pub SequenceOf<AnonymousUEContextModificationFailureProtocolIEs>,
    );
    aper_sequence_of!(
        UEContextModificationFailureProtocolIEs,
        AnonymousUEContextModificationFailureProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE CONTEXT MODIFICATION FAILURE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UEContextModificationFailure {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UEContextModificationFailureProtocolIEs,
    }
    impl UEContextModificationFailure {
        pub fn new(protocol_ies: UEContextModificationFailureProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUEContextModificationRequestProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUEContextModificationRequestProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UEContextModificationRequestProtocolIEs(
        pub SequenceOf<AnonymousUEContextModificationRequestProtocolIEs>,
    );
    aper_sequence_of!(
        UEContextModificationRequestProtocolIEs,
        AnonymousUEContextModificationRequestProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE Context Modification Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE CONTEXT MODIFICATION REQUEST"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UEContextModificationRequest {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UEContextModificationRequestProtocolIEs,
    }
    impl UEContextModificationRequest {
        pub fn new(protocol_ies: UEContextModificationRequestProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUEContextModificationResponseProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUEContextModificationResponseProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UEContextModificationResponseProtocolIEs(
        pub SequenceOf<AnonymousUEContextModificationResponseProtocolIEs>,
    );
    aper_sequence_of!(
        UEContextModificationResponseProtocolIEs,
        AnonymousUEContextModificationResponseProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE CONTEXT MODIFICATION RESPONSE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UEContextModificationResponse {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UEContextModificationResponseProtocolIEs,
    }
    impl UEContextModificationResponse {
        pub fn new(protocol_ies: UEContextModificationResponseProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUEContextReleaseCommandProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUEContextReleaseCommandProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UEContextReleaseCommandProtocolIEs(pub SequenceOf<AnonymousUEContextReleaseCommandProtocolIEs>);
    aper_sequence_of!(UEContextReleaseCommandProtocolIEs, AnonymousUEContextReleaseCommandProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE Context Release Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE CONTEXT RELEASE COMMAND"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UEContextReleaseCommand {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UEContextReleaseCommandProtocolIEs,
    }
    impl UEContextReleaseCommand {
        pub fn new(protocol_ies: UEContextReleaseCommandProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUEContextReleaseCompleteProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUEContextReleaseCompleteProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UEContextReleaseCompleteProtocolIEs(pub SequenceOf<AnonymousUEContextReleaseCompleteProtocolIEs>);
    aper_sequence_of!(UEContextReleaseCompleteProtocolIEs, AnonymousUEContextReleaseCompleteProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE CONTEXT RELEASE COMPLETE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UEContextReleaseComplete {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UEContextReleaseCompleteProtocolIEs,
    }
    impl UEContextReleaseComplete {
        pub fn new(protocol_ies: UEContextReleaseCompleteProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUEContextReleaseRequestProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUEContextReleaseRequestProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UEContextReleaseRequestProtocolIEs(pub SequenceOf<AnonymousUEContextReleaseRequestProtocolIEs>);
    aper_sequence_of!(UEContextReleaseRequestProtocolIEs, AnonymousUEContextReleaseRequestProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE Context Release Request Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE CONTEXT RELEASE REQUEST"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UEContextReleaseRequest {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UEContextReleaseRequestProtocolIEs,
    }
    impl UEContextReleaseRequest {
        pub fn new(protocol_ies: UEContextReleaseRequestProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUERadioCapabilityCheckRequestProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUERadioCapabilityCheckRequestProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UERadioCapabilityCheckRequestProtocolIEs(
        pub SequenceOf<AnonymousUERadioCapabilityCheckRequestProtocolIEs>,
    );
    aper_sequence_of!(
        UERadioCapabilityCheckRequestProtocolIEs,
        AnonymousUERadioCapabilityCheckRequestProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE Radio Capability Check Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE RADIO CAPABILITY CHECK REQUEST"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UERadioCapabilityCheckRequest {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UERadioCapabilityCheckRequestProtocolIEs,
    }
    impl UERadioCapabilityCheckRequest {
        pub fn new(protocol_ies: UERadioCapabilityCheckRequestProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUERadioCapabilityCheckResponseProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUERadioCapabilityCheckResponseProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UERadioCapabilityCheckResponseProtocolIEs(
        pub SequenceOf<AnonymousUERadioCapabilityCheckResponseProtocolIEs>,
    );
    aper_sequence_of!(
        UERadioCapabilityCheckResponseProtocolIEs,
        AnonymousUERadioCapabilityCheckResponseProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE RADIO CAPABILITY CHECK RESPONSE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UERadioCapabilityCheckResponse {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UERadioCapabilityCheckResponseProtocolIEs,
    }
    impl UERadioCapabilityCheckResponse {
        pub fn new(protocol_ies: UERadioCapabilityCheckResponseProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUERadioCapabilityInfoIndicationProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUERadioCapabilityInfoIndicationProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UERadioCapabilityInfoIndicationProtocolIEs(
        pub SequenceOf<AnonymousUERadioCapabilityInfoIndicationProtocolIEs>,
    );
    aper_sequence_of!(
        UERadioCapabilityInfoIndicationProtocolIEs,
        AnonymousUERadioCapabilityInfoIndicationProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE RADIO CAPABILITY MANAGEMENT ELEMENTARY PROCEDURES"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE RADIO CAPABILITY INFO INDICATION"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UERadioCapabilityInfoIndication {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UERadioCapabilityInfoIndicationProtocolIEs,
    }
    impl UERadioCapabilityInfoIndication {
        pub fn new(protocol_ies: UERadioCapabilityInfoIndicationProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUETNLABindingReleaseRequestProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUETNLABindingReleaseRequestProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UETNLABindingReleaseRequestProtocolIEs(pub SequenceOf<AnonymousUETNLABindingReleaseRequestProtocolIEs>);
    aper_sequence_of!(
        UETNLABindingReleaseRequestProtocolIEs,
        AnonymousUETNLABindingReleaseRequestProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE TNLA BINDING ELEMENTARY PROCEDURES"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UE TNLA BINDING RELEASE REQUEST"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UETNLABindingReleaseRequest {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UETNLABindingReleaseRequestProtocolIEs,
    }
    impl UETNLABindingReleaseRequest {
        pub fn new(protocol_ies: UETNLABindingReleaseRequestProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUplinkNASTransportProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUplinkNASTransportProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UplinkNASTransportProtocolIEs(pub SequenceOf<AnonymousUplinkNASTransportProtocolIEs>);
    aper_sequence_of!(UplinkNASTransportProtocolIEs, AnonymousUplinkNASTransportProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UPLINK NAS TRANSPORT"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UplinkNASTransport {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UplinkNASTransportProtocolIEs,
    }
    impl UplinkNASTransport {
        pub fn new(protocol_ies: UplinkNASTransportProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUplinkNonUEAssociatedNRPPaTransportProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUplinkNonUEAssociatedNRPPaTransportProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UplinkNonUEAssociatedNRPPaTransportProtocolIEs(
        pub SequenceOf<AnonymousUplinkNonUEAssociatedNRPPaTransportProtocolIEs>,
    );
    aper_sequence_of!(
        UplinkNonUEAssociatedNRPPaTransportProtocolIEs,
        AnonymousUplinkNonUEAssociatedNRPPaTransportProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UPLINK NON UE ASSOCIATED NRPPA TRANSPORT"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UplinkNonUEAssociatedNRPPaTransport {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UplinkNonUEAssociatedNRPPaTransportProtocolIEs,
    }
    impl UplinkNonUEAssociatedNRPPaTransport {
        pub fn new(protocol_ies: UplinkNonUEAssociatedNRPPaTransportProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUplinkRANConfigurationTransferProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUplinkRANConfigurationTransferProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UplinkRANConfigurationTransferProtocolIEs(
        pub SequenceOf<AnonymousUplinkRANConfigurationTransferProtocolIEs>,
    );
    aper_sequence_of!(
        UplinkRANConfigurationTransferProtocolIEs,
        AnonymousUplinkRANConfigurationTransferProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " CONFIGURATION TRANSFER ELEMENTARY PROCEDURES"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UPLINK RAN CONFIGURATION TRANSFER"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UplinkRANConfigurationTransfer {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UplinkRANConfigurationTransferProtocolIEs,
    }
    impl UplinkRANConfigurationTransfer {
        pub fn new(protocol_ies: UplinkRANConfigurationTransferProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUplinkRANStatusTransferProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUplinkRANStatusTransferProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UplinkRANStatusTransferProtocolIEs(pub SequenceOf<AnonymousUplinkRANStatusTransferProtocolIEs>);
    aper_sequence_of!(UplinkRANStatusTransferProtocolIEs, AnonymousUplinkRANStatusTransferProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Uplink RAN Status Transfer Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UPLINK RAN STATUS TRANSFER"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UplinkRANStatusTransfer {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UplinkRANStatusTransferProtocolIEs,
    }
    impl UplinkRANStatusTransfer {
        pub fn new(protocol_ies: UplinkRANStatusTransferProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousUplinkUEAssociatedNRPPaTransportProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousUplinkUEAssociatedNRPPaTransportProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct UplinkUEAssociatedNRPPaTransportProtocolIEs(
        pub SequenceOf<AnonymousUplinkUEAssociatedNRPPaTransportProtocolIEs>,
    );
    aper_sequence_of!(
        UplinkUEAssociatedNRPPaTransportProtocolIEs,
        AnonymousUplinkUEAssociatedNRPPaTransportProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " UPLINK UE ASSOCIATED NRPPA TRANSPORT"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct UplinkUEAssociatedNRPPaTransport {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: UplinkUEAssociatedNRPPaTransportProtocolIEs,
    }
    impl UplinkUEAssociatedNRPPaTransport {
        pub fn new(protocol_ies: UplinkUEAssociatedNRPPaTransportProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousWriteReplaceWarningRequestProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousWriteReplaceWarningRequestProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct WriteReplaceWarningRequestProtocolIEs(pub SequenceOf<AnonymousWriteReplaceWarningRequestProtocolIEs>);
    aper_sequence_of!(WriteReplaceWarningRequestProtocolIEs, AnonymousWriteReplaceWarningRequestProtocolIEs, 0, 65535);
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " WARNING MESSAGE TRANSMISSION ELEMENTARY PROCEDURES"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Write-Replace Warning Elementary Procedure"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " WRITE-REPLACE WARNING REQUEST"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct WriteReplaceWarningRequest {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: WriteReplaceWarningRequestProtocolIEs,
    }
    impl WriteReplaceWarningRequest {
        pub fn new(protocol_ies: WriteReplaceWarningRequestProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
    #[doc = " Anonymous SEQUENCE OF member "]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags, identifier = "SEQUENCE")]
    pub struct AnonymousWriteReplaceWarningResponseProtocolIEs {
        pub id: ProtocolIEID,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl AnonymousWriteReplaceWarningResponseProtocolIEs {
        pub fn new(id: ProtocolIEID, criticality: Criticality, value: Any) -> Self {
            Self { id, criticality, value }
        }
    }
    #[doc = " Inner type "]
    #[derive(AsnType, Debug, Clone, PartialEq, Eq, Hash)]
    #[rasn(delegate, size("0..=65535"))]
    pub struct WriteReplaceWarningResponseProtocolIEs(pub SequenceOf<AnonymousWriteReplaceWarningResponseProtocolIEs>);
    aper_sequence_of!(
        WriteReplaceWarningResponseProtocolIEs,
        AnonymousWriteReplaceWarningResponseProtocolIEs,
        0,
        65535
    );
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " WRITE-REPLACE WARNING RESPONSE"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    #[non_exhaustive]
    pub struct WriteReplaceWarningResponse {
        #[rasn(identifier = "protocolIEs")]
        pub protocol_ies: WriteReplaceWarningResponseProtocolIEs,
    }
    impl WriteReplaceWarningResponse {
        pub fn new(protocol_ies: WriteReplaceWarningResponseProtocolIEs) -> Self {
            Self { protocol_ies }
        }
    }
}
#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, unused, clippy::too_many_arguments)]
pub mod ngap_pdu_descriptions {
    extern crate alloc;
    use super::ngap_common_data_types::*;
    use super::ngap_constants::*;
    use super::ngap_pdu_contents::*;
    use core::borrow::Borrow;
    use rasn::prelude::*;
    use std::sync::LazyLock;
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct InitiatingMessage {
        #[rasn(identifier = "procedureCode")]
        pub procedure_code: ProcedureCode,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl InitiatingMessage {
        pub fn new(procedure_code: ProcedureCode, criticality: Criticality, value: Any) -> Self {
            Self { procedure_code, criticality, value }
        }
    }
    #[doc = " **************************************************************"]
    #[doc = ""]
    #[doc = " Interface PDU Definition"]
    #[doc = ""]
    #[doc = " **************************************************************"]
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(choice, automatic_tags, identifier = "NGAP-PDU")]
    #[non_exhaustive]
    pub enum NGAPPDU {
        initiatingMessage(InitiatingMessage),
        successfulOutcome(SuccessfulOutcome),
        unsuccessfulOutcome(UnsuccessfulOutcome),
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct SuccessfulOutcome {
        #[rasn(identifier = "procedureCode")]
        pub procedure_code: ProcedureCode,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl SuccessfulOutcome {
        pub fn new(procedure_code: ProcedureCode, criticality: Criticality, value: Any) -> Self {
            Self { procedure_code, criticality, value }
        }
    }
    #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, Eq, Hash)]
    #[rasn(automatic_tags)]
    pub struct UnsuccessfulOutcome {
        #[rasn(identifier = "procedureCode")]
        pub procedure_code: ProcedureCode,
        pub criticality: Criticality,
        pub value: Any,
    }
    impl UnsuccessfulOutcome {
        pub fn new(procedure_code: ProcedureCode, criticality: Criticality, value: Any) -> Self {
            Self { procedure_code, criticality, value }
        }
    }
}
