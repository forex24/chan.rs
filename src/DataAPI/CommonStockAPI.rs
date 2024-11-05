use crate::Common::handle::Handle;
use crate::KLine::KLine_Unit::CKLineUnit;

pub trait CCommonStockApi {
    fn new(
        code: String,
        k_type: String,
        begin_date: String,
        end_date: String,
        autype: String,
    ) -> Self
    where
        Self: Sized;
    fn get_kl_data(&self) -> Box<dyn Iterator<Item = Handle<CKLineUnit>>>;
    fn set_basic_info(&mut self);
    fn do_init();
    fn do_close();
}

pub struct CommonStockApiImpl {
    pub code: String,
    pub name: Option<String>,
    pub is_stock: Option<bool>,
    pub k_type: String,
    pub begin_date: String,
    pub end_date: String,
    pub autype: String,
}

impl CCommonStockApi for CommonStockApiImpl {
    fn new(
        code: String,
        k_type: String,
        begin_date: String,
        end_date: String,
        autype: String,
    ) -> Self {
        let mut api = CommonStockApiImpl {
            code,
            name: None,
            is_stock: None,
            k_type,
            begin_date,
            end_date,
            autype,
        };
        api.set_basic_info();
        api
    }

    fn get_kl_data(&self) -> Box<dyn Iterator<Item = Handle<CKLineUnit>>> {
        // This is a placeholder implementation. You need to implement the actual logic here.
        Box::new(std::iter::empty())
    }

    fn set_basic_info(&mut self) {
        // Implement the logic to set basic info here
    }

    fn do_init() {
        // Implement initialization logic here
    }

    fn do_close() {
        // Implement closing logic here
    }
}

// If you need to implement methods that are specific to CommonStockApiImpl and not part of the trait:
impl CommonStockApiImpl {
    // Add any additional methods here
}
