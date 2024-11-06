#[macro_export]
macro_rules! impl_handle {
    ($name:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+ >)?) => {
        impl $(<$($lt$(:$clt$(+$dlt)*)?),+>)? $crate::AsHandle for $name $(<$($lt),+>)?
        {
            type Output = $crate::Handle<$name $(<$($lt),+ >)?>;
            #[inline(always)]
            fn as_handle(&self) -> Self::Output {
                self.handle
            }
        }

        impl $(<$($lt$(:$clt$(+$dlt)*)?),+>)?  $crate::Indexable for $name $(<$($lt),+>)? {
            #[inline(always)]
            fn index(&self) -> usize {
                self.handle.index()
            }
        }
    }
}
