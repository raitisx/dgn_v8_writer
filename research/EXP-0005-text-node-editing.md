# EXP-0005 Editing text nodes (round 3)

| Field | Value |
| --- | --- |
| Date | 2026-10-14 |
| Performed by | raitisx (licensed MicroStation seat) |
| Product | MicroStation CONNECT Edition (v10), as in EXP-0004 |
| Seed | The empty ADTI LKS-2020 seed, E1 of EXP-0003 (SHA-256 `431af1fd…a757c8`) |
| Files tested | `R06_text_node_edit.dgn`, built with `testfiles/adti_jobs/R06_text_node_edit.json`; SHA-256 `e1aa08b3c345602f097df81727f0899bd8cad9b5cb5f7e1f3c8ee63222ca4a80` |
| Storage | Not in this repository (contains VZD seed content); rebuild as described in EXP-0004 |
| Status | Done. H-N04 promoted to FN-N03, H-N03 to FN-N04. Not reported: save, close and reopen (checklist item 3) |
| Decides | H-N04 (which linkages keep the lines of an edited node apart), H-N03 (node origin for justifications 1 and 2) |

## Why

In EXP-0004 (obs. 5), editing one of our text nodes in MicroStation merged
its two lines into one. MicroStation's own text node in E4 differs from ours
in two linkages that it puts on every text and text node:

- `0x80d4`, 24 bytes, the same on every text except byte 8, which is `0x80`
  on the first line of the node and 0 elsewhere: probably "a line break
  follows". Bytes 12–13 read 1257, which may be the Windows code page of the
  seat that wrote it.
- `0x56f3`, 24 bytes, holding the f64 1.0.

The writer copies these bytes exactly when a job asks for it (H-N04).

## File

All nodes: font 1025, height 0.25 m, line spacing 0.5 m, level Default.

| Node | Position (first line) | Linkages | Justification |
| --- | --- | --- | --- |
| A | 0,10 | none (as in round 2) | 0 |
| B | 10,10 | `0x80d4` on each line, line-break byte set on line 1 | 0 |
| C | 20,10 | both linkages on the node and on each line, exactly as E4 | 0 |
| D | 0,5 | as C | 1 |
| E | 10,5 | as C | 2 |
| F | 20,5 | as C, three Latvian lines | 0 |
| G | 0,0 | a single text, no linkages | 0 |

## Checklist

1. Before editing: Element Information of nodes D and E: what is their
   **Origin**? Expected D 0,4.75 and E 10,4.25 (H-N03).
2. For each node A–F: Edit Text, add `x` at the end of the first line, OK.
   Does the node keep its lines (2, or 3 for F), and do they stay where
   they were?
3. Save, close, reopen: same result?
4. Edit G the same way: does it stay in place?
5. Select any text and open Element Information: what does the **Font**
   field say (the name of font 1025)?

## Results

Tester's report: "R06: A breaks, rest not".

| Node | Linkages | After editing |
| --- | --- | --- |
| A | none | Lines merged ("breaks"), as in EXP-0004 |
| B | `0x80d4` with line-break byte, lines only | Lines kept |
| C | both, as E4 | Lines kept |
| D, E | both, justification 1 and 2 | Lines kept |
| F | both, three Latvian lines | Lines kept |

Follow-up answers (2026-10-14), to the questions in checklist order:

| Check | Answer |
| --- | --- |
| Origins of D and E are 0,4.75 and 10,4.25 | "yes" |
| G stays in place when edited | "yes" |
| Font name of font 1025 | "Arial" |
| (EXP-0004 R04b) the line on the missing level is visible | "visible" |

## Conclusion

The `0x80d4` linkage with the line-break byte is enough to keep the lines
of an edited node apart; `0x56f3` and the linkages on the node header are
not needed for that. H-N04 becomes FN-N03, and the `microstation` profile
now writes `line_breaks` on every text node unless the job says otherwise.
This is exactly node B, so no new test file is needed. The R02 job of
EXP-0004 now says `"linkages": "none"`, so it still rebuilds the file that
was tested.

The node origins for justifications 1 and 2 match the writer's rule, so
H-N03 becomes FN-N04. A single text needs no linkage to stay in place when
edited. Font 1025 is Arial in the ADTI seed.
