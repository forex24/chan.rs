macro_rules! impl_series {
    ($SeriesClassName:ty,$ClassName:ty, $fieldname:ident) => {
        impl $SeriesClassName {
            #[allow(dead_code)]
            #[inline(always)]
            pub fn is_empty(&self) -> bool {
                self.$fieldname.is_empty()
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn len(&self) -> usize {
                self.$fieldname.len()
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn iter(&self) -> std::slice::Iter<'_, $ClassName> {
                self.$fieldname.iter()
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, $ClassName> {
                self.$fieldname.iter_mut()
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn as_slice(&self) -> &[$ClassName] {
                &self.$fieldname.as_slice()
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn as_mut_slice(&mut self) -> &mut [$ClassName] {
                self.$fieldname.as_mut_slice()
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn last(&self) -> Option<&$ClassName> {
                self.$fieldname.last()
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn last_mut(&mut self) -> Option<&mut $ClassName> {
                self.$fieldname.last_mut()
            }
        }

        impl std::ops::Index<usize> for $SeriesClassName {
            type Output = $ClassName;

            fn index(&self, index: usize) -> &Self::Output {
                &self.$fieldname[index]
            }
        }

        impl std::ops::IndexMut<usize> for $SeriesClassName {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                &mut self.$fieldname[index]
            }
        }

        impl std::ops::Index<isize> for $SeriesClassName {
            type Output = $ClassName;

            fn index(&self, index: isize) -> &Self::Output {
                let idx: usize = if index >= 0 {
                    index 
                } else {
                    assert!(index.unsigned_abs() <= self.$fieldname.len());
                    let len = self.$fieldname.len();
                    (len as isize + index) 
                };
                &self.$fieldname[idx]
            }
        }

        impl std::ops::IndexMut<isize> for $SeriesClassName {
            fn index_mut(&mut self, index: isize) -> &mut Self::Output {
                let idx: usize = if index >= 0 {
                    index 
                } else {
                    assert!(index.unsigned_abs() <= self.$fieldname.len());
                    let len = self.$fieldname.len();
                    (len as isize + index) 
                };
                &mut self.$fieldname[idx]
            }
        }

        impl std::ops::Index<i32> for $SeriesClassName {
            type Output = $ClassName;

            fn index(&self, index: i32) -> &Self::Output {
                let idx: usize = if index >= 0 {
                    index 
                } else {
                    assert!(index.unsigned_abs()  <= self.$fieldname.len());
                    let len = self.$fieldname.len();
                    (len as isize + index as isize) 
                };
                &self.$fieldname[idx]
            }
        }

        impl std::ops::IndexMut<i32> for $SeriesClassName {
            fn index_mut(&mut self, index: i32) -> &mut Self::Output {
                let idx: usize = if index >= 0 {
                    index 
                } else {
                    assert!(index.unsigned_abs()  <= self.$fieldname.len());
                    let len = self.$fieldname.len();
                    (len as isize + index as isize) 
                };
                &mut self.$fieldname[idx]
            }
        }

        impl std::ops::Index<std::ops::Range<usize>> for $SeriesClassName {
            type Output = [$ClassName];

            fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
                &self.$fieldname[index.start..index.end]
            }
        }

        impl std::ops::Index<std::ops::RangeFull> for $SeriesClassName {
            type Output = [$ClassName];

            fn index(&self, _index: std::ops::RangeFull) -> &Self::Output {
                self.$fieldname.as_slice()
            }
        }

        impl std::ops::Index<std::ops::RangeFrom<usize>> for $SeriesClassName {
            type Output = [$ClassName];

            fn index(&self, index: std::ops::RangeFrom<usize>) -> &Self::Output {
                &self.$fieldname[index.start..]
            }
        }

        impl std::ops::Index<std::ops::RangeTo<usize>> for $SeriesClassName {
            type Output = [$ClassName];

            fn index(&self, index: std::ops::RangeTo<usize>) -> &Self::Output {
                &self.$fieldname[..index.end]
            }
        }

        impl std::ops::Index<std::ops::RangeToInclusive<usize>> for $SeriesClassName {
            type Output = [$ClassName];

            fn index(&self, index: std::ops::RangeToInclusive<usize>) -> &Self::Output {
                &self.$fieldname[..=index.end]
            }
        }

        impl std::ops::Index<std::ops::RangeInclusive<usize>> for $SeriesClassName {
            type Output = [$ClassName];

            fn index(&self, index: std::ops::RangeInclusive<usize>) -> &Self::Output {
                &self.$fieldname[*index.start()..=*index.end()]
            }
        }
    };
}

macro_rules! impl_handle {
    ($ClassName:ty) => {
        impl crate::AsHandle for $ClassName {
            type Output = Handle<$ClassName>;
            #[inline(always)]
            fn as_handle(&self) -> Self::Output {
                self.handle
            }
        }

        impl crate::Indexable for $ClassName {
            #[inline(always)]
            fn index(&self) -> usize {
                self.handle.index()
            }
        }
    };
}

macro_rules! impl_parent {
    ($ClassName:ty, $ParentClassName:ty, $fieldname:ident) => {
        impl crate::HasParent for $ClassName {
            type Parent = $ParentClassName;

            fn parent(&self) -> Option<Handle<Self::Parent>> {
                self.$fieldname
            }

            fn set_parent(&mut self, parent: Handle<Self::Parent>) {
                self.$fieldname = Some(parent)
            }
        }
    };
}
