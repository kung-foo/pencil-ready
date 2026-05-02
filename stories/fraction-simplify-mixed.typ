#import "/lib/problems/fraction/simplification.typ": fraction-simplification-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Improper fraction that reduces *and* converts to a mixed number:
// 20/6 → 10/3 → 3 1/3. Covers the "reduced before mixed" code path.
#fraction-simplification-problem((20, 6), mode: "worked", debug: true)
