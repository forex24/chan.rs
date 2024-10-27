use num_derive::FromPrimitive;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum ErrCode {
    // chan err
    ChanErrBegin = 0,
    CommonError = 1,
    SrcDataNotFound = 3,
    SrcDataTypeErr = 4,
    ParaError = 5,
    ExtraKluErr = 6,
    SegEndValueErr = 7,
    SegEigenErr = 8,
    BiErr = 9,
    CombinerErr = 10,
    PlotErr = 11,
    ModelError = 12,
    SegLenErr = 13,
    EnvConfErr = 14,
    UnknownDbType = 15,
    FeatureError = 16,
    ConfigError = 17,
    SrcDataFormatError = 18,
    ChanErrEnd = 99,

    // Trade Error
    TradeErrBegin = 100,
    SignalExisted = 101,
    RecordNotExist = 102,
    RecordAlreadyOpened = 103,
    QuotaNotEnough = 104,
    RecordNotOpened = 105,
    TradeUnlockFail = 106,
    PlaceOrderFail = 107,
    ListOrderFail = 108,
    CandelOrderFail = 109,
    GetFutuPriceFail = 110,
    GetFutuLotSizeFail = 111,
    OpenRecordNotWatching = 112,
    GetHoldingQtyFail = 113,
    RecordClosed = 114,
    RequestTradingDaysFail = 115,
    CoverOrderIdNotUnique = 116,
    SignalTraded = 117,
    TradeErrEnd = 199,

    // KL data Error
    KlErrBegin = 200,
    PriceBelowZero = 201,
    KlDataNotAlign = 202,
    KlDataInvalid = 203,
    KlTimeInconsistent = 204,
    TradeinfoTooMuchZero = 205,
    KlNotMonotonous = 206,
    SnapshotErr = 207,
    Suspension = 208, // 疑似停牌
    StockIpoTooLate = 209,
    NoData = 210,
    StockNotActive = 211,
    StockPriceNotActive = 212,
    KlErrEnd = 299,
}

#[derive(Debug)]
pub struct CChanException {
    pub errcode: ErrCode,
    pub msg: String,
}

impl CChanException {
    pub fn new(message: String, code: ErrCode) -> Self {
        CChanException {
            errcode: code,
            msg: message,
        }
    }

    pub fn is_kldata_err(&self) -> bool {
        (ErrCode::KlErrBegin as i32) < (self.errcode as i32)
            && (self.errcode as i32) < (ErrCode::KlErrEnd as i32)
    }

    pub fn is_chan_err(&self) -> bool {
        (ErrCode::ChanErrBegin as i32) < (self.errcode as i32)
            && (self.errcode as i32) < (ErrCode::ChanErrEnd as i32)
    }
}

impl fmt::Display for CChanException {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.errcode as i32, self.msg)
    }
}

impl Error for CChanException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chan_exception() {
        let e = CChanException::new("Config error".to_string(), ErrCode::ConfigError);
        assert_eq!(e.errcode, ErrCode::ConfigError);
        assert_eq!(e.msg, "Config error");
        assert!(e.is_chan_err());
        assert!(!e.is_kldata_err());

        let e2 = CChanException::new("KL data invalid".to_string(), ErrCode::KlDataInvalid);
        assert_eq!(e2.errcode, ErrCode::KlDataInvalid);
        assert_eq!(e2.msg, "KL data invalid");
        assert!(!e2.is_chan_err());
        assert!(e2.is_kldata_err());
    }
}
