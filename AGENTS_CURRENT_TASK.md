# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/installer`

### Architecture Plan
Modify sparringly as new information is learned. Keep minimal and simple. The goal is to always have the architecture in mind while working on a task, as not to go adrift into minefields. The editable area is below:

----------------------

- Edit me if necessary

## Current Tasklist (Remove as things are completed, add remaining tangible tasks)
(If no tasks are listed here, audit the current task and any relevant test cases)

## Important Information
Important information discovered during work about the current state of the task should be appended here.

- We are to fix the installer build, while also migrating from Slint to egui.
- We should keep the slint files around until the very end to use as a visual reference.
- A perfect 1:1 visual migration is not necessary (ie we can use different button visuals), but the general structure should be respected.
- It is critical to use the squalr/ repo as a guide for how to handle egui rendering (ie `app.rs`, `main_window_view.rs`, `project_selector_view.rs`, etc.)
- The installer can be substantially more lightweight. We do not need docking, for instance.

## Agent Scratchpad and Notes
Smaller notes should go here, and can be erased and compacted as needed during iteration.

- It is currently unclear if it makes sense to attempt to share elements (buttons, title bar, footer) between the `squalr` project, and `squalr-installer`. The alternative is just to maintain a copy. This may be acceptable given that the number of total elements is small (header, footer, progress bar, buttons), however maintaining two themes can be tedius.

### Concise Session Log (append-or-compact-only, keep very short and compact as it grows)
