# EXP-0006 Levels added by the writer (round 4)

| Field | Value |
| --- | --- |
| Date | _pending_ |
| Performed by | raitisx (licensed MicroStation seat) |
| Product | MicroStation CONNECT Edition (v10), as in EXP-0004 |
| Seed | The empty ADTI LKS-2020 seed, E1 of EXP-0003 (SHA-256 `431af1fd…a757c8`) |
| Files tested | `R07_new_levels.dgn`, built with `testfiles/adti_jobs/R07_new_levels.json`; SHA-256 `dbaf5124db740bfd3fdad61db2560f8acee77cfa673ff0f743021d8d957f1461` |
| Storage | Not in this repository (contains VZD seed content); rebuild as described in EXP-0004 |
| Status | Pending |
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

_pending_
