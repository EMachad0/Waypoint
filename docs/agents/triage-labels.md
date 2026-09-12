# Triage labels

The skills speak in terms of five canonical triage roles. This file maps those roles to the label
strings this repo's issue tracker actually uses.

| Role in the skills | String in our tracker | Meaning                                  |
| ------------------ | --------------------- | ---------------------------------------- |
| `needs-triage`     | `needs-triage`        | Maintainer needs to evaluate this issue  |
| `needs-info`       | `needs-info`          | Waiting on reporter for more information |
| `ready-for-agent`  | `ready-for-agent`     | Fully specified, ready for an AFK agent  |
| `ready-for-human`  | `ready-for-human`     | Requires human implementation            |
| `wontfix`          | `wontfix`             | Will not be actioned                     |

When a skill names a role, for example "apply the AFK-ready triage label", use the string from the
middle column.

The tracker records these as a `Status:` line in the issue file, not as tracker-native labels. See
`docs/agents/issue-tracker.md`.

## Lifecycle states

The same `Status:` line carries where an issue is in its life. `/waypoint` writes these; the triage
skills only ever write the roles above.

| State        | Meaning                                                         |
| ------------ | --------------------------------------------------------------- |
| `claimed`    | A session is working it, set before the work starts             |
| `resolved`   | Answered or built, with the answer recorded on the issue        |
| `superseded` | A later ticket replaced the answer, named in a `Superseded by:` |
| `wontfix`    | Ruled out of scope, or abandoned                                |

`wontfix` sits in both tables and means the same thing in each. Nothing is deleted when it closes,
so the `Status:` line is the only thing that says an issue is done.
