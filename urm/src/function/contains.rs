use crate::builder::{Build, QueryBuilder};
use crate::database::Database;
use crate::lower::{Lower, Lowered};
use crate::ty::{Nullable, ScalarTyped, Type, Typed, VectorTyped};

/// Binary function that tests equality between two given operands
pub struct Contains<V, I>(pub V, pub I);

impl<DB, V, I> Typed<DB> for Contains<V, I>
where
    DB: Database,
    V: Lower<DB> + Build<DB> + VectorTyped<DB, <V::Ty as Type>::Unit>,
    I: Lower<DB> + Build<DB> + ScalarTyped<DB, <V::Ty as Type>::Unit>,
{
    type Ty = Nullable<bool>;
}

#[cfg(feature = "postgres")]
mod postgres {
    use super::*;
    use crate::database::Postgres;

    impl<V, I> Lower<Postgres> for Contains<V, I>
    where
        V: Lower<Postgres> + Build<Postgres> + VectorTyped<Postgres, <V::Ty as Type>::Unit>,
        I: Lower<Postgres> + Build<Postgres> + ScalarTyped<Postgres, <V::Ty as Type>::Unit>,
    {
        fn lower(self) -> Option<Lowered<Postgres>> {
            Some(Lowered::Expr(Box::new(self)))
        }
    }

    #[cfg(feature = "postgres")]
    impl<V, I> Build<Postgres> for Contains<V, I>
    where
        V: Lower<Postgres> + Build<Postgres> + VectorTyped<Postgres, <V::Ty as Type>::Unit>,
        I: Lower<Postgres> + Build<Postgres> + ScalarTyped<Postgres, <V::Ty as Type>::Unit>,
    {
        fn build(&self, builder: &mut QueryBuilder<Postgres>) {
            self.1.build(builder);
            builder.push(" = any(");
            self.0.build(builder);
            builder.push(")");
        }
    }
}
