# Parent operation caveats retained during round110

- The first combined-review launch request was rejected before task creation:
  the provider accepts `reasoningEffort`, not the requested `effort` option.
  The parent refreshed capabilities after that error and launched the one review
  with `reasoningEffort=xhigh` on the same provider/model. This was not a verifier
  application, completed review, provider switch, or second review chain.
- A parent source read attempted lane A's `ack_cfg.rs` directly under the lane
  root and returned a missing-path error. The checked dependency copies are in
  the candidate's invocation/publication directories; the failed read supplies
  no source or proof evidence. No candidate file was modified to repair it.
- Some long terminal reads were display-truncated. Parent content integrity
  claims rely on byte-reading authentication/archive comparison, not an assertion
  that every displayed source/log line was manually inspected.
- The first parent review-retention execution exited 1 before copying files:
  its line 48 compared the review's empty mismatch lists with integer zero.
  The check was corrected to require empty lists and rerun; this was a parent
  receipt-schema error, not authentication drift or a verifier application.
- No additional parent verifier application was launched during candidate
  authentication or the native atomic source observation. The two reserved
  parent applications remain available only for small review findings within
  the original round deadlines, not for a fresh native probe or unlimited retry.
- Frozen GOAL/STATUS/PLAN/INPUTS remain unchanged while the implementations and
  combined reviewer are active. The live round README and later parent notes
  are separate context, not silently substituted frozen input versions.

These caveats add no proof, native acceptance, trust, or final-cell credit.
