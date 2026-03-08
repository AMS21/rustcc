---
name: Batch fuzzing failure
about: Batch fuzzing run failed and needs investigation
title: ClusterFuzzLite Batch Fuzzing Failure {{ date | date('dddd, MMMM Do yyyy') }}
labels: 'fuzzing'
assignees: 'AMS21'

---

The action {{ action }} failed the fuzzer run.
Run:  [{{ env.RUN_ID }}](https://github.com/AMS21/rustcc/actions/runs/{{ env.RUN_ID }})

- [ ] Failure acknowledged.
- [ ] Failure reproduced with the run's test case artifact.
- [ ] Failure minimized using `cargo fuzz tmin`.
- [ ] Issue fixed.
