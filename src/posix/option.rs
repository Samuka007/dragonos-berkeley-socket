use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[allow(non_camel_case_types)]
pub enum Options {
    DEBUG = 1,
    REUSEADDR = 2,
    TYPE = 3,
    ERROR = 4,
    DONTROUTE = 5,
    BROADCAST = 6,
    SNDBUF = 7,
    RCVBUF = 8,
    SNDBUFFORCE = 32,
    RCVBUFFORCE = 33,
    KEEPALIVE = 9,
    OOBINLINE = 10,
    NO_CHECK = 11,
    PRIORITY = 12,
    LINGER = 13,
    BSDCOMPAT = 14,
    REUSEPORT = 15,
    PASSCRED = 16,
    PEERCRED = 17,
    RCVLOWAT = 18,
    SNDLOWAT = 19,
    RCVTIMEO_OLD = 20,
    SNDTIMEO_OLD = 21,
    SECURITY_AUTHENTICATION = 22,
    SECURITY_ENCRYPTION_TRANSPORT = 23,
    SECURITY_ENCRYPTION_NETWORK = 24,
    BINDTODEVICE = 25,
    /// 与GET_FILTER相同
    ATTACH_FILTER = 26,
    DETACH_FILTER = 27,
    PEERNAME = 28,
    ACCEPTCONN = 30,
    PEERSEC = 31,
    PASSSEC = 34,
    MARK = 36,
    PROTOCOL = 38,
    DOMAIN = 39,
    RXQ_OVFL = 40,
    /// 与SCM_WIFI_STATUS相同
    WIFI_STATUS = 41,
    PEEK_OFF = 42,
    /* Instruct lower device to use last 4-bytes of skb data as FCS */
    NOFCS = 43,
    LOCK_FILTER = 44,
    SELECT_ERR_QUEUE = 45,
    BUSY_POLL = 46,
    MAX_PACING_RATE = 47,
    BPF_EXTENSIONS = 48,
    INCOMING_CPU = 49,
    ATTACH_BPF = 50,
    // DETACH_BPF = DETACH_FILTER,
    ATTACH_REUSEPORT_CBPF = 51,
    ATTACH_REUSEPORT_EBPF = 52,
    CNX_ADVICE = 53,
    SCM_TIMESTAMPING_OPT_STATS = 54,
    MEMINFO = 55,
    INCOMING_NAPI_ID = 56,
    COOKIE = 57,
    SCM_TIMESTAMPING_PKTINFO = 58,
    PEERGROUPS = 59,
    ZEROCOPY = 60,
    /// 与SCM_TXTIME相同
    TXTIME = 61,
    BINDTOIFINDEX = 62,
    TIMESTAMP_OLD = 29,
    TIMESTAMPNS_OLD = 35,
    TIMESTAMPING_OLD = 37,
    TIMESTAMP_NEW = 63,
    TIMESTAMPNS_NEW = 64,
    TIMESTAMPING_NEW = 65,
    RCVTIMEO_NEW = 66,
    SNDTIMEO_NEW = 67,
    DETACH_REUSEPORT_BPF = 68,
    PREFER_BUSY_POLL = 69,
    BUSY_POLL_BUDGET = 70,
    NETNS_COOKIE = 71,
    BUF_LOCK = 72,
    RESERVE_MEM = 73,
    TXREHASH = 74,
    RCVMARK = 75,
}

impl TryFrom<u32> for Options {
    type Error = linux_errnos::Errno;
    fn try_from(x: u32) -> Result<Self, Self::Error> {
        use num_traits::FromPrimitive;
        <Self as FromPrimitive>::from_u32(x).ok_or(Self::Error::EINVAL)
    }
}
