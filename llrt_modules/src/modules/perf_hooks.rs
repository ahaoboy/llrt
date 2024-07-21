// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use crate::ModuleInfo;
use chrono::Utc;
use llrt_utils::module::export_default;
use rquickjs::{
    atom::PredefinedAtom,
    module::{Declarations, Exports, ModuleDef},
    prelude::Func,
    Ctx, Object, Result,
};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

pub static PERFORMANCE_TIME_ORIGIN: AtomicUsize = AtomicUsize::new(0);

fn get_time_origin() -> f64 {
    let time_origin = PERFORMANCE_TIME_ORIGIN.load(Ordering::Relaxed) as f64;

    time_origin / 1e6
}

fn now() -> f64 {
    let now = Utc::now().timestamp_nanos_opt().unwrap_or_default() as f64;
    let started = PERFORMANCE_TIME_ORIGIN.load(Ordering::Relaxed) as f64;

    (now - started) / 1e6
}

fn to_json(ctx: Ctx<'_>) -> Result<Object<'_>> {
    let obj = Object::new(ctx.clone())?;

    obj.set("timeOrigin", get_time_origin())?;

    Ok(obj)
}

pub fn new_performance(ctx: Ctx<'_>) -> Result<Object<'_>> {
    let performance = Object::new(ctx)?;
    performance.set("timeOrigin", get_time_origin())?;
    performance.set("now", Func::from(now))?;
    performance.set(PredefinedAtom::ToJSON, Func::from(to_json))?;
    Ok(performance)
}

pub struct PerfHooksModule;

impl ModuleDef for PerfHooksModule {
    fn declare(declare: &Declarations<'_>) -> Result<()> {
        declare.declare("performance")?;
        declare.declare("default")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        export_default(ctx, exports, |default| {
            let performance = new_performance(ctx.clone())?;
            default.set("performance", performance)?;
            Ok(())
        })
    }
}

impl From<PerfHooksModule> for ModuleInfo<PerfHooksModule> {
    fn from(val: PerfHooksModule) -> Self {
        ModuleInfo {
            name: "perf_hooks",
            module: val,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{call_test, test_async_with, ModuleEvaluator};

    #[tokio::test]
    async fn test_now() {
        test_async_with(|ctx| {
            Box::pin(async move {
                ModuleEvaluator::eval_rust::<PerfHooksModule>(ctx.clone(), "perf_hooks")
                    .await
                    .unwrap();

                let module = ModuleEvaluator::eval_js(
                    ctx.clone(),
                    "test",
                    r#"
                        import { performance } from 'perf_hooks';

                        export async function test() {
                            const now = performance.now()
                            // TODO: Delaying with setTimeout
                            for(let i=0; i < (1<<20); i++){}

                            return performance.now() - now
                        }
                    "#,
                )
                .await
                .unwrap();
                let result = call_test::<u32, _>(&ctx, &module, ()).await;
                assert!(result > 0)
            })
        })
        .await;
    }

    #[tokio::test]
    async fn test_time_origin() {
        test_async_with(|ctx| {
            Box::pin(async move {
                ModuleEvaluator::eval_rust::<PerfHooksModule>(ctx.clone(), "perf_hooks")
                    .await
                    .unwrap();

                let module = ModuleEvaluator::eval_js(
                    ctx.clone(),
                    "test",
                    r#"
                        import { performance } from 'perf_hooks';

                        export async function test() {
                            return performance.timeOrigin
                        }
                    "#,
                )
                .await
                .unwrap();
                let result = call_test::<f64, _>(&ctx, &module, ()).await;
                assert!(result == 0.0);
            })
        })
        .await;
    }
}
