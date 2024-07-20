// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use crate::ModuleInfo;
use llrt_utils::module::export_default;
use once_cell::sync::Lazy;
use rquickjs::{
    module::{Declarations, Exports, ModuleDef},
    prelude::Func,
    Ctx, Object, Result,
};
use std::time::SystemTime;

static START_TIME: Lazy<SystemTime> = Lazy::new(SystemTime::now);

fn now() -> usize {
    let start = *START_TIME;
    SystemTime::now()
        .duration_since(start)
        .expect("Time went backwards")
        .as_millis() as usize
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
            let now_fn = Func::from(now);
            let performance = Object::new(ctx.clone())?;
            performance.set("now", now_fn)?;
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
}
