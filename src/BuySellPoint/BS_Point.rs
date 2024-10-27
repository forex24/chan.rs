use crate::bi::bi::CBi;
use crate::chan_model::features::CFeatures;
use crate::common::c_enum::BspType;
use crate::seg::seg::CSeg;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub enum LineType {
    Bi(Rc<RefCell<CBi>>),
    Seg(Rc<RefCell<CSeg>>),
}

pub struct CBSPoint {
    bi: LineType,
    klu: Rc<RefCell<CKLineUnit>>,
    is_buy: bool,
    bsp_type: Vec<BspType>,
    relate_bsp1: Option<Rc<RefCell<CBSPoint>>>,
    features: CFeatures,
    is_segbsp: bool,
}

impl CBSPoint {
    pub fn new(
        bi: LineType,
        is_buy: bool,
        bs_type: BspType,
        relate_bsp1: Option<Rc<RefCell<CBSPoint>>>,
        feature_dict: Option<HashMap<String, f64>>,
    ) -> Rc<RefCell<Self>> {
        let klu = match &bi {
            LineType::Bi(b) => b.borrow().get_end_klu(),
            LineType::Seg(s) => s.borrow().get_end_klu(),
        };

        let bsp = Rc::new(RefCell::new(CBSPoint {
            bi,
            klu,
            is_buy,
            bsp_type: vec![bs_type],
            relate_bsp1,
            features: CFeatures::new(feature_dict),
            is_segbsp: false,
        }));

        match &bsp.borrow().bi {
            LineType::Bi(b) => b.borrow_mut().bsp = Some(Rc::clone(&bsp)),
            LineType::Seg(s) => s.borrow_mut().bsp = Some(Rc::clone(&bsp)),
        }

        bsp.borrow_mut().init_common_feature();

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
        relate_bsp1: Option<Rc<RefCell<CBSPoint>>>,
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
        let amp = match &self.bi {
            LineType::Bi(b) => b.borrow().amp(),
            LineType::Seg(s) => s.borrow().amp(),
        };

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
