#import "/lib/problems/factors/gcf.typ": gcf-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Worked: factors listed, common ones ringed, 1 included. 60 and 96
// have the longest lists the generator allows; 8 and 15 share only 1.
#stack(spacing: 0.5em,
  gcf-problem((12, 18, 6), mode: "worked", debug: true),
  gcf-problem((60, 96, 12), mode: "worked", debug: true),
  gcf-problem((8, 15, 1), mode: "worked", debug: true),
)
