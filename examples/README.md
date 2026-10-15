# Examples

Two job files to start from. Paths in a job are relative to the job file.

| Job | Seed | What it shows |
| --- | --- | --- |
| `gdal_example.json` | GDAL's public test file (bundled) | Runs anywhere: line, circle, Latvian text, text node, cell. GDAL's model is 3D, hence `"allow_3d_model": true`, and it uses the round-1 `"oda"` profile. |
| `adti_example.json` | The ADTI LKS-2020 seed, which you supply | The real use: levels added to the seed, ByLevel symbology, LKS-92 coordinates, Latvian text, a text node and a rotated cell. |

Run them:

```text
dgnv8w build examples/gdal_example.json
dgnv8w build examples/adti_example.json     # needs the ADTI seed, see below
```

The ADTI seed is not in this repository. In the repository, copy it to
`private/adti_seed.dgn` (git ignores `private/`). In a release bundle,
put it next to the job as `adti_seed.dgn`.

The full job format is in [docs/JOB_FORMAT.md](../docs/JOB_FORMAT.md).
