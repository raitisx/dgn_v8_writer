# EXP-0006 Levels added by the writer (round 4)

| Field | Value |
| --- | --- |
| Date | 2026-10-14 |
| Performed by | raitisx (licensed MicroStation seat) |
| Product | MicroStation CONNECT Edition (v10), as in EXP-0004 |
| Seed | The empty ADTI LKS-2020 seed, E1 of EXP-0003 (SHA-256 `431af1fd…a757c8`) |
| Files tested | `R07_new_levels.dgn`, built with `testfiles/adti_jobs/R07_new_levels.json`; SHA-256 `dbaf5124db740bfd3fdad61db2560f8acee77cfa673ff0f743021d8d957f1461` |
| Storage | Not in this repository (contains VZD seed content); rebuild as described in EXP-0004 |
| Status | Done: added levels work in MicroStation. Open: the number of a level created in Level Manager (obs. 4) |
| Decides | FN-V03 (adding a level by copying the seed's "Default" entry), H-V04 (non-ASCII level names) |

## Why

The user asked the writer to add levels itself rather than rely on a seed
with the ADTI levels imported. The only MicroStation-made new level we have
is the one in E4, which MicroStation copied from the ADTI library and
changed in fields we do not understand. The writer instead copies the
seed's own "Default" entry and changes only its IDs, time, name and
description (FN-V03).

## File

Four new levels, written into the seed's level table (`Dgn^Nm/$4`) before
"Default":

| Level ID | Name | Description | Elements on it |
| --- | --- | --- | --- |
| 1 | ZZ_TEST_A | Pārbaudes līmenis A | Line 0,0–10,0 (colour 3); the components of cell ADTI_TEST at 14,3 |
| 2 | ZZ_TEST_B | — | Shape at 0,2–4,4 (colour 4); two-line text node at 0,8 (colour 6) |
| 3 | ZZ_LĪMENIS_Ā | Latviešu burti nosaukumā | Text "ZZ_LĪMENIS_Ā" at 6,2 (colour 5) |
| 4 | ZZ_UNUSED | Bez elementiem | none |
| 64 | Default (existing) | | Line 0,−2–10,−2 (colour 1) |

The cell header stays on level 0, as MicroStation's own cells do (FN-C01).
The new levels inherit Default's settings, so their ByLevel colour, weight
and style are Default's.

## Checklist

1. Does the file open without messages?
2. Level Manager: are ZZ_TEST_A, ZZ_TEST_B, ZZ_LĪMENIS_Ā and ZZ_UNUSED
   listed with these names, numbers 1–4 and descriptions, Latvian letters
   included?
3. Element Information of each element: is the Level as in the table?
4. Level Display: switch ZZ_TEST_B off. Do the shape and the text node
   disappear, and nothing else?
5. Level Manager: delete ZZ_UNUSED. Is that allowed? Then create a new
   level: what number does it get?
6. Place a line on ZZ_TEST_A, save, close, reopen: are all levels and
   elements still there?

Optional, for a later step (each level's own colour, weight and style):
copy E1, create one level in Level Manager named `ZZ_EVID` with colour 3,
weight 2 and line style 2, save it as E6, and upload it like E1–E4.

## Results

Tester's answers, in checklist order: "1. yes 2. yes 3. yes. yes. yes.
4. yes 5. yes. no number. 6. yes".

| Check | Result |
| --- | --- |
| 1. Opens | Yes, without messages |
| 2. Level Manager lists the four levels with numbers 1–4, names and descriptions, Latvian letters included | Yes |
| 3. Elements on the expected levels (ZZ_TEST_A; ZZ_TEST_B; ZZ_LĪMENIS_Ā) | Yes, yes, yes |
| 4. Switching ZZ_TEST_B off hides only the shape and the text node | Yes |
| 5. Delete ZZ_UNUSED; number of a level created afterwards | Deleting is allowed; the new level shows "no number" |
| 6. Line placed on ZZ_TEST_A, save, close, reopen | Everything still there |

## Observations

1. MicroStation CONNECT v10 accepts levels added as copies of the seed's
   "Default" entry: it lists them, uses them for display, element
   assignment and level display, and keeps them through its own save
   (FN-V03).
2. Names and descriptions with Latvian letters, written as `ff fe` +
   UTF-16LE, display correctly (H-V04, now FN-V04).
3. A copied level can be deleted, so the copy does not carry a "Default
   cannot be deleted" protection.
4. A level created in Level Manager afterwards shows "no number". Level
   Manager's number column may be a separate level code rather than the ID
   at 0x20; which ID MicroStation gave that level is not known yet. The
   file the tester saved in step 6 would show it, and how MicroStation
   writes a level of its own.
