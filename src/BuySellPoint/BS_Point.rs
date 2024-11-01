use crate::ChanModel::Features::CFeatures;
use crate::Common::types::Handle;
use crate::Common::CEnum::BspType;
use crate::KLine::KLine_Unit::CKLineUnit;
use crate::Seg::linetype::Line;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct CBSPoint<T> {
    pub bi: Handle<T>,
    pub klu: Handle<CKLineUnit>,
    pub is_buy: bool,
    pub bsp_type: Vec<BspType>,
    pub relate_bsp1: Option<Handle<CBSPoint<T>>>,
    pub features: CFeatures,
    pub is_segbsp: bool,
}

impl<T: Line> CBSPoint<T> {
    pub fn new(
        bi: Handle<T>,
        is_buy: bool,
        bs_type: BspType,
        relate_bsp1: Option<Handle<CBSPoint<T>>>,
        feature_dict: Option<HashMap<String, Option<f64>>>,
    ) -> Self {
        let klu = bi.borrow().get_end_klu();

        let mut bsp = CBSPoint {
            bi,
            klu,
            is_buy,
            bsp_type: vec![bs_type],
            relate_bsp1,
            features: CFeatures::new(feature_dict),
            is_segbsp: false,
        };

        bi.borrow_mut().bsp = Some(Rc::clone(&bsp));

        bsp.init_common_feature();

        bsp
    }

    pub fn add_type(&mut self, bs_type: BspType) {
        self.bsp_type.push(bs_type);
    }

    pub fn type2str(&self) -> String {
        self.bsp_type
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }

    pub fn add_another_bsp_prop(
        &mut self,
        bs_type: BspType,
        relate_bsp1: Option<Handle<CBSPoint<T>>>,
    ) {
        self.add_type(bs_type);
        if self.relate_bsp1.is_none() {
            self.relate_bsp1 = relate_bsp1;
        } else if let Some(new_relate_bsp1) = relate_bsp1 {
            assert_eq!(
                self.relate_bsp1.as_ref().unwrap().borrow().klu.borrow().idx,
                new_relate_bsp1.borrow().klu.borrow().idx
            );
        }
    }

    pub fn add_feat(&mut self, inp1: FeatureInput, inp2: Option<f64>) {
        self.features.add_feat(inp1, inp2);
    }

    fn init_common_feature(&mut self) {
        let amp = self.bi.borrow().amp();

        self.add_feat(
            FeatureInput::Dict(HashMap::from([("bsp_bi_amp".to_string(), amp)])),
            None,
        );
    }
}

pub enum FeatureInput {
    String(String),
    Dict(HashMap<String, f64>),
    DictOpt(HashMap<String, Option<f64>>),
    Features(CFeatures),
}
