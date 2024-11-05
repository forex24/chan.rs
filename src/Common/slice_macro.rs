/*#[macro_export]
macro_rules! impl_series {
    ($SeriesClassName:ident $(<$($slt:tt$(:$sclt:tt$(+$sdlt:tt)*)?),+ >)?,$ClassName:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+ >)?, $fieldname:ident) => {
        impl$(<$($slt$(:$sclt$(+$sdlt)*)?),+>)?   $SeriesClassName $(<$($slt),+>)? {
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
            pub fn iter(&self) -> std::slice::Iter<'_, $ClassName $(<$($lt),+>)?> {
                self.$fieldname.iter()
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, $ClassName $(<$($lt),+>)?> {
                self.$fieldname.iter_mut()
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn as_slice(&self) -> &[$ClassName $(<$($lt),+>)?] {
                &self.$fieldname.as_slice()
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn as_mut_slice(&mut self) -> &mut [$ClassName $(<$($lt),+>)?] {
                self.$fieldname.as_mut_slice()
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn last(&self) -> Option<&$ClassName $(<$($lt),+>)?> {
                self.$fieldname.last()
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn last_mut(&mut self) -> Option<&mut $ClassName $(<$($lt),+>)?> {
                self.$fieldname.last_mut()
            }
        }

        impl$(<$($slt$(:$sclt$(+$sdlt)*)?),+>)?  std::ops::Index<usize> for  $SeriesClassName $(<$($slt),+>)?  {
            type Output = $ClassName $(<$($lt),+>)?;

            fn index(&self, index: usize) -> &Self::Output {
                &self.$fieldname[index]
            }
        }

        impl$(<$($slt$(:$sclt$(+$sdlt)*)?),+>)?  std::ops::IndexMut<usize> for  $SeriesClassName $(<$($slt),+>)?  {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                &mut self.$fieldname[index]
            }
        }

        impl$(<$($slt$(:$sclt$(+$sdlt)*)?),+>)?  std::ops::Index<isize> for  $SeriesClassName $(<$($slt),+>)?  {
            type Output = $ClassName $(<$($lt),+>)?;

            fn index(&self, index: isize) -> &Self::Output {
                let idx: usize = if index >= 0 {
                    index as usize
                } else {
                    assert!((index.unsigned_abs() as usize) <= self.$fieldname.len());
                    let len = self.$fieldname.len();
                    (len as isize + index) as usize
                };
                &self.$fieldname[idx]
            }
        }

        impl$(<$($slt$(:$sclt$(+$sdlt)*)?),+>)?  std::ops::IndexMut<isize> for  $SeriesClassName $(<$($slt),+>)?  {
            fn index_mut(&mut self, index: isize) -> &mut Self::Output {
                let idx: usize = if index >= 0 {
                    index as usize
                } else {
                    assert!((index.unsigned_abs()) <= self.$fieldname.len());
                    let len = self.$fieldname.len();
                    (len as isize + index) as usize
                };
                &mut self.$fieldname[idx]
            }
        }

        impl$(<$($slt$(:$sclt$(+$sdlt)*)?),+>)?  std::ops::Index<i32> for  $SeriesClassName $(<$($slt),+>)?  {
            type Output = $ClassName $(<$($lt),+>)?;

            fn index(&self, index: i32) -> &Self::Output {
                let idx: usize = if index >= 0 {
                    index as usize
                } else {
                    assert!((index.unsigned_abs() as usize) <= self.$fieldname.len());
                    let len = self.$fieldname.len();
                    (len as isize + index as isize) as usize
                };
                &self.$fieldname[idx]
            }
        }

        impl$(<$($slt$(:$sclt$(+$sdlt)*)?),+>)?  std::ops::IndexMut<i32> for  $SeriesClassName $(<$($slt),+>)?  {
            fn index_mut(&mut self, index: i32) -> &mut Self::Output {
                let idx: usize = if index >= 0 {
                    index as usize
                } else {
                    assert!((index.unsigned_abs() as usize) <= self.$fieldname.len());
                    let len = self.$fieldname.len();
                    (len as isize + index as isize) as usize
                };
                &mut self.$fieldname[idx]
            }
        }

        impl$(<$($slt$(:$sclt$(+$sdlt)*)?),+>)?  std::ops::Index<std::ops::Range<usize>> for  $SeriesClassName $(<$($slt),+>)?  {
            type Output = [$ClassName $(<$($lt),+>)?];

            fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
                &self.$fieldname[index.start..index.end]
            }
        }

        impl$(<$($slt$(:$sclt$(+$sdlt)*)?),+>)?  std::ops::Index<std::ops::RangeFull> for  $SeriesClassName $(<$($slt),+>)?  {
            type Output = [$ClassName $(<$($lt),+>)?];

            fn index(&self, _index: std::ops::RangeFull) -> &Self::Output {
                self.$fieldname.as_slice()
            }
        }

        impl$(<$($slt$(:$sclt$(+$sdlt)*)?),+>)?  std::ops::Index<std::ops::RangeFrom<usize>> for  $SeriesClassName $(<$($slt),+>)?  {
            type Output = [$ClassName $(<$($lt),+>)?];

            fn index(&self, index: std::ops::RangeFrom<usize>) -> &Self::Output {
                &self.$fieldname[index.start..]
            }
        }

        impl$(<$($slt$(:$sclt$(+$sdlt)*)?),+>)?  std::ops::Index<std::ops::RangeTo<usize>> for  $SeriesClassName $(<$($slt),+>)?  {
            type Output = [$ClassName $(<$($lt),+>)?];

            fn index(&self, index: std::ops::RangeTo<usize>) -> &Self::Output {
                &self.$fieldname[..index.end]
            }
        }

        impl$(<$($slt$(:$sclt$(+$sdlt)*)?),+>)?  std::ops::Index<std::ops::RangeToInclusive<usize>> for  $SeriesClassName $(<$($slt),+>)?  {
            type Output = [$ClassName $(<$($lt),+>)?];

            fn index(&self, index: std::ops::RangeToInclusive<usize>) -> &Self::Output {
                &self.$fieldname[..=index.end]
            }
        }

        impl$(<$($slt$(:$sclt$(+$sdlt)*)?),+>)?  std::ops::Index<std::ops::RangeInclusive<usize>> for  $SeriesClassName $(<$($slt),+>)?  {
            type Output = [$ClassName $(<$($lt),+>)?];

            fn index(&self, index: std::ops::RangeInclusive<usize>) -> &Self::Output {
                &self.$fieldname[*index.start()..=*index.end()]
            }
        }
    };
}*/

#[macro_export]
macro_rules! impl_handle {
    ($name:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+ >)?) => {
        impl $(<$($lt$(:$clt$(+$dlt)*)?),+>)? $crate::Common::handle::AsHandle for $name $(<$($lt),+>)?
        {
            type Output = Handle<$name $(<$($lt),+ >)?>;
            #[inline(always)]
            fn as_handle(&self) -> Self::Output {
                self.handle
            }
        }

        impl $(<$($lt$(:$clt$(+$dlt)*)?),+>)?  $crate::Common::handle::Indexable for $name $(<$($lt),+>)? {
            #[inline(always)]
            fn index(&self) -> usize {
                self.handle.idx
            }
        }
    }
}

#[macro_export]
macro_rules! impl_parent {
    (($ClassName:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+>)?), ($ParentClassName:ident $(<$($plt:tt$(:$pclt:tt$(+$pdlt:tt)*)?),+>)?), $fieldname:ident) => {
        impl$(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+>)?  $crate::HasParent for $ClassName $(<$($lt),+>)? {
            type Parent = $ParentClassName $(<$($plt:tt$(:$pclt:tt$(+$pdlt:tt)*)?),+>)?;

            fn parent(&self) -> Option<Handle<Self::Parent>> {
                self.$fieldname
            }

            fn set_parent(&mut self, parent: Handle<Self::Parent>) {
                self.$fieldname = Some(parent)
            }
        }
    };
}
