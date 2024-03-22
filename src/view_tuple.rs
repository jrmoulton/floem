use crate::view::View;
use crate::view::Widget;

pub trait IntoWidgets {
    fn into_widgets(self) -> Vec<Box<dyn Widget>>;
}

// Implement IntoWidgets for any type that implements View
impl<T: View + 'static> IntoWidgets for T {
    fn into_widgets(self) -> Vec<Box<dyn Widget>> {
        vec![self.build()]
    }
}

// Implement IntoWidgets for Vec<Box<dyn Widget>>
impl IntoWidgets for Vec<Box<dyn Widget>> {
    fn into_widgets(self) -> Vec<Box<dyn Widget>> {
        self
    }
}

// Macro to implement ViewTuple for tuples of Views and Vec<Box<dyn Widget>>
macro_rules! impl_view_tuple {
    ($($t:ident),+) => {
        impl<$($t: IntoWidgets + 'static),+> IntoWidgets for ($($t,)+) {
            fn into_widgets(self) -> Vec<Box<dyn Widget>> {
                #[allow(non_snake_case)]
                let ($($t,)+) = self;
                let mut widgets = Vec::new();
                $(
                    widgets.extend($t.into_widgets());
                )+
                widgets
            }
        }
    };
}

impl_view_tuple!(A);
impl_view_tuple!(A, B);
impl_view_tuple!(A, B, C);
impl_view_tuple!(A, B, C, D);
impl_view_tuple!(A, B, C, D, E);
impl_view_tuple!(A, B, C, D, E, F);
impl_view_tuple!(A, B, C, D, E, F, G);
impl_view_tuple!(A, B, C, D, E, F, G, H);
impl_view_tuple!(A, B, C, D, E, F, G, H, I);
impl_view_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_view_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_view_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_view_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_view_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_view_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_view_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
