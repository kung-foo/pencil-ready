import type { WorksheetKind } from "@/lib/api";

export type WorksheetInfo = {
  /** Plain-English title, matches the server's title() output. */
  title: string;
  /** One-sentence description of what the worksheet drills. */
  summary: string;
  /** What the student should already be comfortable with. */
  prerequisites: string[];
  /** Skills gained with mastery of this worksheet. */
  learning: string[];
};

export const WORKSHEET_INFO: Record<WorksheetKind, WorksheetInfo> = {
  add: {
    title: "Addition",
    summary: "Add whole numbers in columns, carrying when a column sums past 9.",
    prerequisites: [
      "Count reliably up to at least 20.",
      "Recognize digits 0–9 and know each digit's place value (ones, tens, hundreds).",
      "Understand the “+” symbol as combining two quantities.",
    ],
    learning: [
      "Line up numbers by place value and add column-by-column from right to left.",
      "Carry the tens digit when a column sum reaches 10 or more.",
      "Check a result by re-reading the carried digits in each column.",
    ],
  },
  subtract: {
    title: "Subtraction",
    summary: "Subtract the smaller number from the larger, borrowing across columns when needed.",
    prerequisites: [
      "Column addition (adding in place).",
      "Counting backwards and understanding the inverse relationship with addition.",
      "Recognize when the top digit is smaller than the bottom digit.",
    ],
    learning: [
      "Column subtraction with place-value alignment.",
      "Borrowing 10 from the next column and crossing out correctly.",
      "Handling borrows that ripple across consecutive zeros (hardest case).",
    ],
  },
  multiply: {
    title: "Multiplication",
    summary: "Multiply multi-digit numbers using partial products and column-alignment.",
    prerequisites: [
      "Fluency with single-digit times tables (0–10 minimum).",
      "Column addition, including carrying.",
      "Understanding multiplication as repeated addition.",
    ],
    learning: [
      "Partial-products layout: one product row per digit of the multiplier.",
      "Shifting each partial product one place left for every digit up.",
      "Summing the partial products to arrive at the final answer.",
    ],
  },
  "simple-divide": {
    title: "Division",
    summary: "Recall times tables in reverse — given the product and a factor, find the other factor.",
    prerequisites: [
      "Confident multiplication facts up to at least 10×10.",
      "Understanding multiplication and division as inverse operations.",
    ],
    learning: [
      "Division as “how many groups of N fit in M”.",
      "Rapid recall of quotients derived from known times tables.",
      "Foundation for long division — clean evenly-dividing cases first.",
    ],
  },
  "long-divide": {
    title: "Long division",
    summary: "Divide larger numbers step-by-step using the divide–multiply–subtract–bring-down loop.",
    prerequisites: [
      "Simple division (times-table-based).",
      "Multi-digit subtraction with borrowing.",
      "Keeping aligned columns on lined or grid paper helps.",
    ],
    learning: [
      "The four-step cycle: Divide, Multiply, Subtract, Bring down.",
      "Deciding the first digit of the quotient — does the divisor “fit” the leading digits?",
      "Tracking remainders column-by-column, ending with a final remainder (or zero).",
    ],
  },
  "mult-drill": {
    title: "Multiplication drill",
    summary: "Horizontal facts practice for rapid recall of specific times tables.",
    prerequisites: [
      "Understanding of multiplication as repeated addition.",
      "Introduction to at least one table (usually 2s or 10s).",
    ],
    learning: [
      "Fluency — recall times tables within a few seconds without counting.",
      "Commutative property: 7 × 3 = 3 × 7. The drill deduplicates pairs.",
      "Building confidence before tackling multi-digit multiplication.",
    ],
  },
  "div-drill": {
    title: "Division drill",
    summary: "Horizontal facts practice mirroring the multiplication drill, in reverse.",
    prerequisites: [
      "Multiplication drill fluency for the corresponding tables.",
      "Understanding that division undoes multiplication.",
    ],
    learning: [
      "Recognize every product in a times table and name the matching quotient.",
      "Prepare for simple and long division worksheets with automatic recall.",
    ],
  },
  "fraction-mult": {
    title: "Fraction multiplication",
    summary: "Multiply a whole number by a proper fraction, producing a whole-number answer.",
    prerequisites: [
      "Multi-digit multiplication.",
      "Read a fraction as numerator over denominator.",
      "Simple division, including noticing when a product divides evenly.",
    ],
    learning: [
      "Multiply across: (whole × numerator) / denominator.",
      "Simplify to a whole number when the denominator divides the numerator.",
      "Recognize which (whole, fraction) pairs produce clean whole answers.",
    ],
  },
  "algebra-two-step": {
    title: "Two-step equations",
    summary: "Solve ax + b = c for x by isolating the variable.",
    prerequisites: [
      "Arithmetic facts (addition, subtraction, multiplication, division).",
      "Understand that a letter like x is a placeholder for an unknown number.",
      "Comfortable with inverse operations (subtract to undo addition, divide to undo multiplication).",
    ],
    learning: [
      "Isolate the variable by undoing operations in reverse order: subtract the constant, then divide by the coefficient.",
      "Keep the equation balanced — whatever you do on one side, do on the other.",
      "Read three canonical forms: (ax) + b = c, b + (ax) = c, (ax) − b = c.",
    ],
  },
};
