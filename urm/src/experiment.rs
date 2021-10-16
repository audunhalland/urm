//!
//! Experimental modelling of the SQL abstraction
//!
//! We have to be able to build in a completely hierarchical manner.
//! Which is not easy, because there may be data dependency between
//! outer and inner expressions..
//!
//! Example:
//!
//! ```sql
//! SELECT
//!   json_build_object(
//!     'f', foo.f,
//!     'b_list', bar_json,
//!   ) AS json,
//! FROM
//!   foo
//! JOIN LATERAL (
//!   SELECT
//!     jsonb_agg(
//!       json_build_object(
//!         'b', bar.b
//!       )
//!     )
//!   FROM
//!     bar
//!   WHERE
//!     bar.foo_id = foo.id
//! ) bar_json ON TRUE
//!
//! ```
