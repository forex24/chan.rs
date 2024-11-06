use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::option::Option;
use std::rc::Rc;
use std::vec::Vec;

use crate::AsHandle;
use crate::Bar;
use crate::BspType;
use crate::CFeatures;
use crate::Handle;
use crate::IBspInfo;
use crate::IParent;
use crate::LineType;

// 买卖点
#[derive(Debug)]
pub struct CBspPoint<T> {
    pub bi: Handle<T>,
    pub klu: Handle<Bar>,
    pub is_buy: bool,
    pub types: Vec<BspType>,

    pub relate_bsp1: Option<Rc<RefCell<CBspPoint<T>>>>,
    pub features: CFeatures,
    pub is_segbsp: bool,
}

impl<T: LineType + IBspInfo> CBspPoint<T> {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::borrowed_box)]
    pub fn new(
        bi: Handle<T>,
        is_buy: bool,
        bs_type: BspType,
        relate_bsp1: Option<Rc<RefCell<CBspPoint<T>>>>,
        feature_dict: Option<HashMap<String, Option<f64>>>,
    ) -> Rc<RefCell<Self>> {
        let bsp_point = Rc::new(RefCell::new(Self {
            bi,
            klu: bi.get_end_klu().as_handle(),
            is_buy,
            types: vec![bs_type],
            relate_bsp1,
            features: CFeatures::new(feature_dict),
            is_segbsp: false,
        }));
        bsp_point
            .borrow_mut()
            .bi
            .as_mut()
            .set_bsp(bsp_point.clone());
        bsp_point
    }

    // Add a new type to the types vector
    fn add_type(&mut self, bs_type: BspType) {
        self.types.push(bs_type);
    }

    // Add another BSP property
    pub(crate) fn add_another_bsp_prop(
        &mut self,
        bs_type: BspType,
        relate_bsp1: Option<Rc<RefCell<CBspPoint<T>>>>,
    ) {
        self.add_type(bs_type);
        if self.relate_bsp1.is_none() {
            self.relate_bsp1 = relate_bsp1;
        } else if let Some(ref relate_bsp1) = relate_bsp1 {
            assert_eq!(
                self.relate_bsp1
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .klu
                    .as_handle()
                    .index(),
                relate_bsp1.borrow().klu.as_handle().index()
            );
        }
    }
}

impl<T> CBspPoint<T> {
    pub fn type2str(&self) -> String {
        self.types
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("_")
    }
    // TODO:Add a feature
    //pub fn add_feat(&mut self, inp1: impl Into<FeaturesInput>, inp2: Option<f64>) {
    //    self.features.add_feat(inp1, inp2);
    //}
}

impl<T: LineType + IParent> Display for CBspPoint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "bi:{} klu:{} is_buy:{} is_segbsp:{} types:{} rbsp1:{:?}",
            self.bi.index(),
            self.klu.index(),
            self.is_buy,
            self.is_segbsp,
            self.type2str(),
            self.relate_bsp1.as_ref().map(|x| x.borrow().bi.index()),
        )
    }
}
