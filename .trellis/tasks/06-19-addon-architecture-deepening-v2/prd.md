---
date: 2026-06-19
topic: addon-architecture-deepening-v2
title: Addon architecture deepening v2
origin: docs/workstreams/addon-architecture-deepening/README.md
---

## Summary

继续深化 Nako 的 Addon 控制平面 seam，把当前集中在 `crates/nako-server/src/app/addons.rs` 和 `crates/nako-server/src/http/addons.rs` 的职责拆成更小的 concept module。目标是让注册、token/grant、runtime health、hosted surfaces、task runtime、resource search、subtitle workflow 各自通过更清晰的 interface 被测试和维护。

## Problem Frame

Addon 方向本身已经成立，问题在于 depth 还不够。当前上层 module 仍然把多种 Addon 生命周期、诊断、路由翻译和会话状态揉在一起，caller 学到的 interface 过宽，tests 也不得不围着巨大的 orchestrator 走。对后续 Addon breadth 来说，这种形态会让权限、目标校验、redaction、task 运行和搜索/字幕流程继续外溢到调用方。

本次工作不是重新讨论 Addon Manager，也不是恢复已关闭的 lifecycle automation 方向。它只深化当前 Addon 控制平面 seam，让后续 Addon 能力扩展继续落在可测试、可分隔的 module 上。

## Requirements

### Addon app module 深化

R1. Addon app 需要拆成更小的 concept module，让 registration、token/grant、runtime health、hosted surfaces、task runtime、resource search、subtitle workflow 的规则各自局部化。

R2. Addon app 的上层入口要保持薄，只负责把请求转发到对应的 concept module，不再承担多条 Addon 流程的编排知识。

R3. 共享的 principal、target、routing、redaction 和 scope 规则要收口到明确的 seam，避免它们散落在多个 Addon 子模块里。

### HTTP translation layer 深化

R4. Addon HTTP 路由层要继续保持 thin translation，只做请求/响应映射，不再承载 Addon 领域规则。

R5. `/admin/v1/addons/*` 下的 route family 要按职责继续分组，使路由测试能直接命中各自的 concept module seam。

### 可测试性与边界

R6. 每个被拆出的 concept module 都要有足够小的 interface，让测试优先跨 seam，而不是穿透到大 orchestrator 的内部细节。

R7. 本次变更不能重新引入已关闭的 addon-manager lifecycle automation，也不能把 Addon 运行时收回到 in-process plugin 方向。

## Scope Boundaries

**Deferred for later**

- Addon Manager discovery、install、update、marketplace、package signing、process supervision。
- 更广的 Addon Task runtime、Event Subscription delivery、subtitle breadth、library file write breadth。

**Outside this product's identity**

- Native Plugin ABI。
- Jellyfin Plugin Compatibility。
- 把 Addon 运行时改回进程内插件模型。

## Dependencies / Assumptions

- 现有 Addon 协议和 Admin API 契约保持稳定。
- 现有 Addon workstream 的已关闭边界继续有效。
- 现有测试可以作为拆分后的回归网。

## Success Criteria

- Addon 控制平面的职责分布更清楚，`app/addons.rs` 和 `http/addons.rs` 不再是单点高耦合入口。
- 新增或修改某一类 Addon 行为时，通常只需要改一个 concept module。
- 相关测试能更直接地落在对应 seam 上，而不是通过大而宽的 orchestrator 间接验证。
- 代码结构更接近后续 Addon breadth 的自然落点。

## Acceptance Notes

- 任务执行时应优先保留现有行为，再逐步下沉 module 深度。
- 若某个候选拆分不能明显提高 locality 或测试 leverage，就不要保留。
