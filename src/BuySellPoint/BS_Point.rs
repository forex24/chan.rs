use crate::ChanModel::Features::CFeatures;
use crate::ChanModel::Features::FeatureInput;
use crate::Common::types::{Handle, WeakHandle};
use crate::Common::CEnum::BspType;
use crate::KLine::KLine_Unit::CKLineUnit;
use crate::Seg::linetype::Line;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct CBSPoint<T> {
    pub bi: WeakHandle<T>,
    pub klu: WeakHandle<CKLineUnit>,
    pub is_buy: bool,
    pub bsp_type: Vec<BspType>,
    pub relate_bsp1: Option<WeakHandle<CBSPoint<T>>>,
    pub features: CFeatures,
    pub is_segbsp: bool,
}

impl<T: Line> CBSPoint<T> {
    pub fn new(
        bi: WeakHandle<T>,
        is_buy: bool,
        bs_type: BspType,
        relate_bsp1: Option<WeakHandle<CBSPoint<T>>>,
        feature_dict: Option<HashMap<String, Option<f64>>>,
    ) -> Handle<Self> {
        let klu = Rc::downgrade(&bi.upgrade().unwrap().borrow().line_get_end_klu());
        let bi_clone = bi.clone();

        let features = match feature_dict {
            Some(dict) => {
                let flattened = dict
                    .into_iter()
                    .filter_map(|(k, v)| v.map(|val| (k, Some(val))))
                    .collect();
                CFeatures::new(Some(flattened))
            }
            None => CFeatures::new(None),
        };

        let bsp = Rc::new(RefCell::new(CBSPoint {
            bi,
            klu,
            is_buy,
            bsp_type: vec![bs_type],
            relate_bsp1,
            features,
            is_segbsp: false,
        }));

        bi_clone
            .upgrade()
            .unwrap()
            .borrow_mut()
            .line_set_bsp(Some(bsp.clone()));

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
        relate_bsp1: Option<WeakHandle<CBSPoint<T>>>,
    ) {
        self.add_type(bs_type);
        if self.relate_bsp1.is_none() {
            self.relate_bsp1 = relate_bsp1;
        } else if let Some(new_relate_bsp1) = relate_bsp1 {
            assert_eq!(
                self.relate_bsp1
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .klu
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .idx,
                new_relate_bsp1
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .klu
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .idx
            );
        }
    }

    pub fn add_feat(&mut self, inp1: impl Into<FeatureInput>) {
        self.features.add_feat(inp1);
    }

    fn init_common_feature(&mut self) {
        let amp = self.bi.upgrade().unwrap().borrow().line_amp();

        self.add_feat(("bsp_bi_amp", amp));
    }
}
