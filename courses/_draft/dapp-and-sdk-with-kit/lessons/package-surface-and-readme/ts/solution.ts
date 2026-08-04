/**
 * Classify a client change as a semver bump.
 *
 * Inputs are pipe-joined descriptors: the instruction names and the account
 * field names, before and after the change. Empty string means "none".
 *   e.g. beforeInstrs = "deposit|withdraw", afterFields = "owner|balance|bump"
 *
 * The most breaking change is decided first: breakage (removed field, changed
 * layout, removed instruction) outranks addition, or an added instruction would
 * mask a layout change and a "minor" bump would break a consumer's build.
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

  if (fieldsChanged || instrRemoved) return "major"; // (B) breakage first
  if (instrAdded) return "minor"; // (A) additive
  return "patch"; // (D) identical surface, regenerated
}
