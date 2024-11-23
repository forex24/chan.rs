use std::collections::HashMap;
use std::fmt::Display;
use std::option::Option;

use std::vec::Vec;

use crate::AsHandle;
use crate::Bar;
use crate::BspType;
use crate::CFeatures;
use crate::Handle;
use crate::IBspInfo;
use crate::IParent;
use crate::LineType;
use smallvec::{smallvec, SmallVec};

// 买卖点
#[derive(Debug)]
pub struct CBspPoint<T> {
    pub handle: Handle<Self>,
    pub bi: Handle<T>,
    pub klu: Handle<Bar>,
    pub is_buy: bool,
    pub types: SmallVec<[BspType; 4]>,

    pub relate_bsp1: Option<Handle<CBspPoint<T>>>,
    pub features: CFeatures,
    pub is_segbsp: bool,
}

impl<T: LineType + IBspInfo> CBspPoint<T> {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::borrowed_box)]
    pub fn new(
        boxed_vec: &Box<Vec<Self>>,
        idx: usize,
        bi: Handle<T>,
        is_buy: bool,
        bs_type: BspType,
        relate_bsp1: Option<Handle<CBspPoint<T>>>,
        feature_dict: Option<HashMap<String, Option<f64>>>,
    ) -> Self {
        let bsp = Self {
            handle: Handle::new(boxed_vec, idx),
            bi,
            klu: bi.get_end_klu(),
            is_buy,
            types: smallvec![bs_type],
            relate_bsp1,
            features: CFeatures::new(feature_dict),
            is_segbsp: false,
        };

        bsp.bi.as_mut().set_bsp(bsp.as_handle());

        //bsp.borrow_mut().init_common_feature();

        bsp
    }

    // Add a new type to the types vector
    fn add_type(&mut self, bs_type: BspType) {
        self.types.push(bs_type);
    }

    // Add another BSP property
    pub(crate) fn add_another_bsp_prop(
        &mut self,
        bs_type: BspType,
        relate_bsp1: Option<Handle<CBspPoint<T>>>,
    ) {
        self.add_type(bs_type);

        if self.relate_bsp1.is_none() {
            self.relate_bsp1 = relate_bsp1;
        } else if let Some(ref relate_bsp1) = relate_bsp1 {
            assert_eq!(
                self.relate_bsp1.as_ref().unwrap().klu.as_handle().index(),
                relate_bsp1.klu.as_handle().index()
            );
        }
    }

    //pub fn add_feat<K>(&mut self, inp1: K, inp2: Option<f64>)
    //where
    //    K: Into<CFeatures> + std::fmt::Debug,
    //{
    //    self.features.add_feat(inp1, inp2);
    //}
    //
    //pub fn init_common_feature(&mut self) {
    //    // Initialize features that apply to all buy/sell points
    //    let mut common_features = HashMap::new();
    //    self.features
    //        .insert("bsp_bi_amp".to_string(), self.bi.amp());
    //
    //    self.add_feat(common_features, None);
    //}
}

impl<T> CBspPoint<T> {
    pub fn type2str(&self) -> String {
        let types_str = self.types.iter().map(|x| x.to_string()).collect::<Vec<_>>();

        if self.types.len() == 1 {
            types_str.join(",") // 单一值，不加引号
        } else {
            format!("\"{}\"", types_str.join(",")) // 多个值，加引号
        }
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
            self.relate_bsp1.as_ref().map(|x| x.bi.index()),
        )
    }
}

impl_handle!(CBspPoint<T>);
