# EXP-0003 Evidence files made in MicroStation

| Field | Value |
| --- | --- |
| Date | _pending_ |
| Performed by | raitisx (licensed MicroStation seat) |
| MicroStation product and version | _fill in_ |
| Seed used | _VZD LKS-2020 ADTI seed, file name and date_ |
| Storage | Private repository (the files contain VZD seed content and must not go into this public repository). Only hashes and observations are recorded here. |
| Status | Pending |
| Decides | FN-M02, FN-P01 (MicroStation page format), FN-S02/H-H02, H-T03/H-T04, H-N01, FN-C01, H-C02, level and font tables |

## Protocol

Minimal pairs: each file differs from the previous one by one action, so a
byte diff shows exactly what that action writes. Step-by-step instructions
are in [`testfiles/README.md`](../testfiles/README.md#part-2-make-evidence-files-in-microstation).

| File | Content |
| --- | --- |
| E1_empty.dgn | New file from the ADTI seed, saved with nothing drawn |
| E2_line.dgn | E1 + one line (0,0)–(10,5) |
| E3_text.dgn | E2 + one Latvian text |
| E4_cell_node.dgn | E3 + one ADTI cell + one two-line text node |
| E5_many.dgn (optional) | E4 + about 3000 copied lines |

## Records

| File | Bytes | SHA-256 | Element IDs noted in MicroStation |
| --- | --- | --- | --- |
| E1_empty.dgn | | | n/a |
| E2_line.dgn | | | |
| E3_text.dgn | | | |
| E4_cell_node.dgn | | | |

## Observations

_pending_
