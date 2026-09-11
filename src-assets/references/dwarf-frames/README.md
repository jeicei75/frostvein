# Dwarf reference frames — the good ones, named by what each is authoritative for

**Why this directory exists.** The model sheet's proportions were all measured from **one frame** of
`dwarf.mp4` — t=10 s — because that is what an `fps=1` sample happened to hand me. It turns out to
be among the **worst** frames in the file: dim, three-quarter, and heavily compressed. The video is
240 frames and contains brightly-lit near-front AND near-side views. Round 4's flat side profile,
its featureless head and its missing depth are all downstream of measuring from the wrong frame, and
that is a defect in the input, not in the modelling.

Extracted with `ffmpeg -i dwarf.mp4 all/%03d.png`, cropped `380x620+360+90`, no rescaling. `fNNN` is
the original frame number.

| frame | view | authoritative for |
|---|---|---|
| `f088.png` | **near-pure SIDE** | **depth — every Y dimension.** The pack's projection, the arm's thickness, the leg profile, the boot length. Round 4 estimated all of these. |
| `f084.png` | side, one step earlier | the **domed, stepped crown** — the head is NOT a flat-topped box; the hair silhouette steps down at the back |
| `f092.png` | side, one step later | cross-check for `f088`; boot and heel shape |
| `f104.png` | **near-FRONT, brightly lit** | **front proportions and the face.** Replaces t=10 s as the front measuring surface |
| `f100.png` | near-front | cross-check for `f104`; tunic value range |
| `f140.png` | three-quarter, lit | **the neck** — visible between beard and collar — and the **shoulder caps** |
| `f164.png` | three-quarter, lit, mid-swing | **shoulders** again, plus how the sleeve sits on the shoulder |
| `f120.png` | side, bent over | the back's structure behind the pack, and the tunic hem in profile |

## What these frames settle that the old one could not

- **The head has form.** `f084`/`f088` show a stepped, domed crown. A single box is the minimum-effort
  reading of the reference, not a faithful one.
- **There is a neck.** `f140`/`f164`. Round 4 seats the head directly on the torso.
- **There are shoulders.** `f140`/`f164` — the sleeve sits ON the shoulder and the tunic is wider at
  the top than the torso beneath it. Wolf's note that "the reference is lacking those" was drawn
  from t=10 s, where they are not visible; in these frames they are.
- **The boots are long** in profile — `f084`–`f092`.
- **The tunic is a brighter, more saturated green than t=10 s suggested.** That frame was dim, so
  part of the "the reference is warmer than our palette" question was a LIGHTING artifact of the
  frame it was asked about. The palette decision stays gated on Epic 11 regardless.

## Still missing, and worth saying

**There is no pure BACK view in the video.** `f084`–`f092` and `f120` show the pack from behind-side,
which is enough to derive structure but not enough to measure. The back therefore remains partly
invented — an orthographic turnaround is still the single cheapest input that would fix it, and it
is the one thing none of this can synthesise.

Every frame here is still **lit, in perspective, and compressed.** They are much better than t=10 s;
they are not an orthographic sheet. Widths are foreshortened, albedo is not recoverable, and the
±4 % caveat in the model sheet still applies.
