/**
 * Converts a score value (-100 to 100) into a descriptive string
 * @param score A number between -100 and 100
 * @returns A string describing the focus level
 */
export function getScoreDescription(score: number): string {
  if (score < -80) return "Highly Distractive";
  if (score < -40) return "Distractive";
  if (score < 0) return "Slightly Distractive";
  if (score === 0) return "Neutral";
  if (score < 40) return "Slightly Focused";
  if (score < 80) return "Focused";
  return "Highly Focused";
}
