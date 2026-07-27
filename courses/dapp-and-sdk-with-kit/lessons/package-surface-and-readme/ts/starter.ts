/**
 * Classify a client change as a semver bump.
 *
 * Inputs are pipe-joined descriptors: the instruction names and the account
 * field names, before and after the change. Empty string means "none".
 *   e.g. beforeInstrs = "deposit|withdraw", afterFields = "owner|balance|bump"
 *
 * PARSONS: the three return branches you need are all present below, plus one
 * decoy. They are in the WRONG ORDER. Reorder them so the most breaking change
 * is decided first — a change that both adds an instruction and alters a layout
 * is a `major`, never a `minor`.
 */
function classifyChange(
  beforeInstrs: string,
  afterInstrs: string,
  beforeFields: string,
  afterFields: string
): "major" | "minor" | "patch" {
  const split = (s: string): string[] => (s === "" ? [] : s.split("|"));
  const bI = split(beforeInstrs);
  const aI = split(afterInstrs);
  const bF = split(beforeFields);
  const aF = split(afterFields);

  const fieldsChanged =
    bF.length !== aF.length ||
    bF.some((f) => !aF.includes(f)) ||
    aF.some((f) => !bF.includes(f));
  const instrRemoved = bI.some((i) => !aI.includes(i));
  const instrAdded = aI.some((i) => !bI.includes(i));

  // --- reorder the four lines below ---------------------------------------
  if (instrAdded) return "minor"; // (A)
  if (fieldsChanged || instrRemoved) return "major"; // (B)
  // if (aF.length > bF.length) return "minor"; // (C) DECOY — an added field is a layout change, i.e. major
  return "patch"; // (D)
}
