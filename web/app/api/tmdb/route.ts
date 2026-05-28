import { NextResponse } from "next/server"

const TMDB_API_KEY = process.env.TMDB_API_KEY || "demo"
const TMDB_BASE_URL = "https://api.themoviedb.org/3"
const TMDB_IMAGE_BASE = "https://image.tmdb.org/t/p"

const FALLBACK_MOVIES = [
  {
    id: 693134,
    title: "Dune 2",
    original_title: "Dune: Part Two",
    overview: "Paul Atreides unites with Chani and the Fremen.",
    poster_path: "/8b8R8l88Qje9dn9OE8PY05Nxl1X.jpg",
    backdrop_path: "/xOMo8BRK7PfcJv9JCnx7s5hj0PX.jpg",
    release_date: "2024-02-27",
    vote_average: 8.2,
    genre_ids: [878, 12],
    media_type: "movie",
  },
  {
    id: 872585,
    title: "Oppenheimer",
    original_title: "Oppenheimer",
    overview: "The story of American scientist J. Robert Oppenheimer.",
    poster_path: "/8Gxv8gSFCU0XGDykEGv7zR1n2ua.jpg",
    backdrop_path: "/fm6KqXpk3M2HVveHwCrBSSBaO0V.jpg",
    release_date: "2023-07-19",
    vote_average: 8.1,
    genre_ids: [18, 36],
    media_type: "movie",
  },
  {
    id: 157336,
    title: "Interstellar",
    original_title: "Interstellar",
    overview: "A team of explorers travel through a wormhole in space.",
    poster_path: "/gEU2QniE6E77NI6lCU6MxlNBvIx.jpg",
    backdrop_path: "/xJHokMbljvjADYdit5fK5VQsXEG.jpg",
    release_date: "2014-11-05",
    vote_average: 8.4,
    genre_ids: [12, 18, 878],
    media_type: "movie",
  },
  {
    id: 335984,
    title: "Blade Runner 2049",
    original_title: "Blade Runner 2049",
    overview: "Officer K discovers a secret that leads him to find Rick Deckard.",
    poster_path: "/gajva2L0rPYkEWjzgFlBXCAVBE5.jpg",
    backdrop_path: "/ilRyazdMJwN05exqhwK4tMKBYZs.jpg",
    release_date: "2017-10-04",
    vote_average: 7.5,
    genre_ids: [878, 18],
    media_type: "movie",
  },
  {
    id: 329865,
    title: "Arrival",
    original_title: "Arrival",
    overview: "A linguist works with the military to communicate with alien lifeforms.",
    poster_path: "/x2FJsf1ElAgr63Y3PNPtJrcmpoe.jpg",
    backdrop_path: "/yIZ1xendyqKvY3FGeeUYUd5X9Mm.jpg",
    release_date: "2016-11-10",
    vote_average: 7.6,
    genre_ids: [18, 878],
    media_type: "movie",
  },
]

const FALLBACK_SERIES = [
  {
    id: 46648,
    name: "True Detective",
    original_name: "True Detective",
    overview: "An anthology crime drama series.",
    poster_path: "/aowr6lVeNWgxyQYXrOHnqiOxnEq.jpg",
    backdrop_path: "/r3e03VfCJ5ZvIYhJbEScGa7BPgI.jpg",
    first_air_date: "2014-01-12",
    vote_average: 8.3,
    genre_ids: [80, 18],
    media_type: "tv",
  },
  {
    id: 1396,
    name: "Breaking Bad",
    original_name: "Breaking Bad",
    overview: "A high school chemistry teacher turns to manufacturing and selling meth.",
    poster_path: "/ggFHVNu6YYI5L9pCfOacjizRGt.jpg",
    backdrop_path: "/tsRy63Mu5cu8etL1X7ZLyf7UP1M.jpg",
    first_air_date: "2008-01-20",
    vote_average: 8.9,
    genre_ids: [18, 80],
    media_type: "tv",
  },
  {
    id: 76331,
    name: "Succession",
    original_name: "Succession",
    overview: "The Roy family is known for controlling the biggest media company.",
    poster_path: "/7HW47XbkNQ5fiwQFYGWdw9gs144.jpg",
    backdrop_path: "/wYfprynJGc0MQ6lCU6rGVPgGU.jpg",
    first_air_date: "2018-06-03",
    vote_average: 8.5,
    genre_ids: [18],
    media_type: "tv",
  },
]

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url)
  const endpoint = searchParams.get("endpoint") || "trending"
  const query = searchParams.get("query")
  const id = searchParams.get("id")
  const mediaType = searchParams.get("media_type") || "movie"
  
  if (!process.env.TMDB_API_KEY || process.env.TMDB_API_KEY === "demo") {
    return NextResponse.json({
      results: [...FALLBACK_MOVIES, ...FALLBACK_SERIES],
      fallback: true,
      image_base: TMDB_IMAGE_BASE,
    })
  }

  try {
    let url = ""
    
    switch (endpoint) {
      case "trending":
        url = `${TMDB_BASE_URL}/trending/all/week?api_key=${TMDB_API_KEY}&language=zh-CN`
        break
      case "popular_movies":
        url = `${TMDB_BASE_URL}/movie/popular?api_key=${TMDB_API_KEY}&language=zh-CN`
        break
      case "popular_tv":
        url = `${TMDB_BASE_URL}/tv/popular?api_key=${TMDB_API_KEY}&language=zh-CN`
        break
      case "search":
        url = `${TMDB_BASE_URL}/search/multi?api_key=${TMDB_API_KEY}&language=zh-CN&query=${encodeURIComponent(query || "")}`
        break
      case "details":
        url = `${TMDB_BASE_URL}/${mediaType}/${id}?api_key=${TMDB_API_KEY}&language=zh-CN&append_to_response=credits,images,videos`
        break
      default:
        url = `${TMDB_BASE_URL}/trending/all/week?api_key=${TMDB_API_KEY}&language=zh-CN`
    }

    const response = await fetch(url, { next: { revalidate: 3600 } })

    if (!response.ok) {
      throw new Error(`TMDb API error: ${response.status}`)
    }

    const data = await response.json()
    
    return NextResponse.json({
      ...data,
      image_base: TMDB_IMAGE_BASE,
    })
  } catch (error) {
    console.error("TMDb API error:", error)
    return NextResponse.json({
      results: [...FALLBACK_MOVIES, ...FALLBACK_SERIES],
      fallback: true,
      image_base: TMDB_IMAGE_BASE,
    })
  }
}
