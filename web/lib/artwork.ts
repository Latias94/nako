const ARTWORK_MAP: Record<string, string> = {
  "8b8R8l88Qje9dn9OE8PY05Nxl1X.jpg": "/posters/dune2.jpg",
  "d5NXSklXo0qyIYkgV94XAgMIckC.jpg": "/posters/dune2.jpg",
  "aowr4xpLP5sRCL50TkuADomJ98T.jpg": "/posters/true-detective.jpg",
  "7wUxRFB7JVNX5fGobvV6kDaAHE0.jpg": "/posters/true-detective.jpg",
  "aowr6lVeNWgxyQYXrOHnqiOxnEq.jpg": "/posters/true-detective.jpg",
  "8Gxv8gSFCU0XGDykEGv7zR1n2ua.jpg": "/posters/oppenheimer.jpg",
  "gEU2QniE6E77NI6lCU6MxlNBvIx.jpg": "/posters/interstellar.jpg",
  "gajva2L0rPYkEWjzgFlBXCAVBE5.jpg": "/posters/blade-runner.jpg",
  "ggFHVNu6YYI5L9pCfOacjizRGt.jpg": "/posters/breaking-bad.jpg",
  "7HW47XbkNQ5fiwQFYGWdw9gs144.jpg": "/posters/succession.jpg",
  "x2FJsf1ElAgr63Y3PNPtJrcmpoe.jpg": "/posters/arrival.jpg",
  "tRNlZbgNCNOpLpbPEz5L8G8A0JN.jpg": "/posters/blade-runner.jpg",
  "xuAIuYSmsUzKlUMBFGVZaWsY3DZ.jpg": "/avatars/avatar-1.jpg",
  "cGOPbv9wA5gEejkUN892JrveARt.jpg": "/avatars/avatar-2.jpg",
  "k68nPLbIST6NP96JmTxmZijEvCA.jpg": "/avatars/avatar-3.jpg",
  "ebSnODDg9lbsMIaWg2uAbjn7TO5.jpg": "/placeholder-user.jpg",
  "hr0L2aueqlP2BYUblTTjmtn0hw4.jpg": "/placeholder.jpg",
  "edv5CZvWj09upOsy2Y6IwDhK8bt.jpg": "/posters/interstellar.jpg",
  "qJ2tW6WMUDux911r6m7haRef0WH.jpg": "/posters/blade-runner.jpg",
  "dksTL9NXc3GqPBRHYHcy1aIwjS.jpg": "/placeholder-user.jpg",
  "dZm8DQrABIgbERXUWXjb1EPtPcT.jpg": "/placeholder-user.jpg",
  "BE2sdjpgsa2rNTFa66f7upkaOP.jpg": "/avatars/avatar-1.jpg",
  "oIhJnzTxOjFPMPiaS4TslhKD4dx.jpg": "/avatars/avatar-2.jpg",
  "lJloTOheuQSirSLXNA3JHsrMNfH.jpg": "/avatars/avatar-3.jpg",
  "1hDuMBcW1TYpLaGLV6fsnHzxwKs.jpg": "/avatars/avatar-4.jpg",
  "xOMo8BRK7PfcJv9JCnx7s5hj0PX.jpg": "/backdrops/dune2-backdrop.jpg",
  "fm6KqXpk3M2HVveHwCrBSSBaO0V.jpg": "/backdrops/dune2-backdrop.jpg",
  "xJHokMbljvjADYdit5fK5VQsXEG.jpg": "/backdrops/dune2-backdrop.jpg",
  "pEzNVQfdzYDzVK0XqxERcGj0VJg.jpg": "/backdrops/dune2-backdrop.jpg",
  "e2X5hq5sJJVk1gPajaXaeRE57Fp.jpg": "/posters/succession.jpg",
  "49WJfeN0moxb9IPfGn8AIqMGskD.jpg": "/posters/true-detective.jpg",
  "dqZENchTd7lp5zht7BdlqFWjk6H.jpg": "/placeholder.jpg",
  "rUunhF0rKaUJLzBj0wvKrczwqhA.jpg": "/placeholder.jpg",
}

export function resolveArtwork(src?: string | null): string {
  if (!src) {
    return "/placeholder.jpg"
  }

  if (!src.startsWith("http")) {
    return src
  }

  try {
    const filename = new URL(src).pathname.split("/").pop()
    if (!filename) {
      return "/placeholder.jpg"
    }

    return ARTWORK_MAP[filename] || "/placeholder.jpg"
  } catch {
    return "/placeholder.jpg"
  }
}
