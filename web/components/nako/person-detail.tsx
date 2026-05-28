"use client"
import { resolveArtwork } from '@/lib/artwork'

import { useState } from "react"
import { ArrowLeft, Calendar, MapPin, Star, Play, ExternalLink, Instagram, Twitter, Globe, Film, Tv, ChevronRight } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { cn } from "@/lib/utils"

interface PersonDetailProps {
  personId: string
  personName: string
  onBack: () => void
  onSelectMedia: (id: string, type: "movie" | "series") => void
}

// 示例人物数据
const personData = {
  id: "1",
  name: "Christopher Nolan",
  chineseName: "克里斯托弗·诺兰",
  profileImage: "/avatars/avatar-1.jpg",
  birthday: "1970-07-30",
  birthplace: "London, England, UK",
  knownFor: "Directing",
  biography: `Christopher Edward Nolan CBE is a British-American filmmaker known for his Hollywood blockbusters with complex storytelling. He is considered one of the most influential filmmakers of the 21st century.

His films have grossed more than US$6 billion worldwide, and have garnered 11 Academy Awards and 36 nominations. Nolan has been recognized for his work with numerous accolades including being named by Time magazine as one of the 100 most influential people in the world in 2015.

Known for his ambitious and visually striking films, Nolan frequently explores philosophical and ethical concepts, including personal identity, memory, and time.`,
  socialLinks: {
    instagram: "https://instagram.com",
    twitter: "https://twitter.com",
    website: "https://christophernolan.net",
  },
  stats: {
    totalFilms: 12,
    avgRating: 8.4,
    awards: 11,
    nominations: 36,
  },
}

// 示例作品数据
const filmography = {
  director: [
    { id: "1", title: "Oppenheimer", year: 2023, rating: 8.4, poster: "/posters/oppenheimer.jpg", type: "movie", role: "Director" },
    { id: "2", title: "Tenet", year: 2020, rating: 7.3, poster: "/avatars/avatar-3.jpg", type: "movie", role: "Director, Writer" },
    { id: "3", title: "Dunkirk", year: 2017, rating: 7.8, poster: "/placeholder.jpg", type: "movie", role: "Director, Writer" },
    { id: "4", title: "Interstellar", year: 2014, rating: 8.7, poster: "/posters/interstellar.jpg", type: "movie", role: "Director, Writer" },
    { id: "5", title: "The Dark Knight Rises", year: 2012, rating: 8.4, poster: "/placeholder.jpg", type: "movie", role: "Director, Writer" },
    { id: "6", title: "Inception", year: 2010, rating: 8.8, poster: "/placeholder.jpg", type: "movie", role: "Director, Writer" },
    { id: "7", title: "The Dark Knight", year: 2008, rating: 9.0, poster: "/placeholder.jpg", type: "movie", role: "Director, Writer" },
    { id: "8", title: "The Prestige", year: 2006, rating: 8.5, poster: "/posters/blade-runner.jpg", type: "movie", role: "Director, Writer" },
  ],
  writer: [
    { id: "1", title: "Oppenheimer", year: 2023, rating: 8.4, poster: "/posters/oppenheimer.jpg", type: "movie", role: "Writer" },
    { id: "9", title: "Man of Steel", year: 2013, rating: 7.1, poster: "/placeholder.jpg", type: "movie", role: "Story" },
  ],
  producer: [
    { id: "1", title: "Oppenheimer", year: 2023, rating: 8.4, poster: "/posters/oppenheimer.jpg", type: "movie", role: "Producer" },
    { id: "10", title: "Transcendence", year: 2014, rating: 6.2, poster: "/placeholder.jpg", type: "movie", role: "Executive Producer" },
  ],
}

// 示例图片数据
const personImages = [
  { id: "1", url: "/avatars/avatar-1.jpg" },
  { id: "2", url: "/avatars/avatar-2.jpg" },
]

export function PersonDetail({ personId, personName, onBack, onSelectMedia }: PersonDetailProps) {
  const [activeTab, setActiveTab] = useState("filmography")

  const calculateAge = (birthday: string) => {
    const birth = new Date(birthday)
    const today = new Date()
    let age = today.getFullYear() - birth.getFullYear()
    const m = today.getMonth() - birth.getMonth()
    if (m < 0 || (m === 0 && today.getDate() < birth.getDate())) {
      age--
    }
    return age
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-40 border-b border-border/50 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex h-14 items-center gap-4 px-4 lg:px-6">
          <Button variant="ghost" size="icon" onClick={onBack}>
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <h1 className="text-lg font-semibold">{personData.chineseName || personData.name}</h1>
        </div>
      </header>

      <div className="mx-auto max-w-6xl px-4 py-6 lg:px-6 lg:py-8">
        {/* Profile Header */}
        <div className="mb-8 flex flex-col gap-6 md:flex-row md:gap-8">
          {/* Profile Image */}
          <div className="flex-shrink-0">
            <div className="relative mx-auto h-64 w-48 overflow-hidden rounded-xl bg-muted shadow-lg md:mx-0 md:h-80 md:w-60">
              <img
                src={resolveArtwork(personData.profileImage)}
                alt={personData.name}
                className="h-full w-full object-cover"
              />
            </div>
          </div>

          {/* Profile Info */}
          <div className="flex-1 text-center md:text-left">
            <h2 className="text-2xl font-bold lg:text-3xl">{personData.name}</h2>
            {personData.chineseName && (
              <p className="mt-1 text-lg text-muted-foreground">{personData.chineseName}</p>
            )}

            <Badge variant="secondary" className="mt-3">
              {personData.knownFor}
            </Badge>

            {/* Basic Info */}
            <div className="mt-4 flex flex-wrap justify-center gap-4 text-sm text-muted-foreground md:justify-start">
              {personData.birthday && (
                <div className="flex items-center gap-1.5">
                  <Calendar className="h-4 w-4" />
                  <span>{personData.birthday} ({calculateAge(personData.birthday)} years old)</span>
                </div>
              )}
              {personData.birthplace && (
                <div className="flex items-center gap-1.5">
                  <MapPin className="h-4 w-4" />
                  <span>{personData.birthplace}</span>
                </div>
              )}
            </div>

            {/* Stats */}
            <div className="mt-6 grid grid-cols-4 gap-4 rounded-lg bg-secondary/30 p-4">
              <div className="text-center">
                <div className="text-2xl font-bold text-primary">{personData.stats.totalFilms}</div>
                <div className="text-xs text-muted-foreground">Films</div>
              </div>
              <div className="text-center">
                <div className="flex items-center justify-center gap-1 text-2xl font-bold text-accent">
                  <Star className="h-5 w-5 fill-current" />
                  {personData.stats.avgRating}
                </div>
                <div className="text-xs text-muted-foreground">Avg Rating</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-primary">{personData.stats.awards}</div>
                <div className="text-xs text-muted-foreground">Awards</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-muted-foreground">{personData.stats.nominations}</div>
                <div className="text-xs text-muted-foreground">Nominations</div>
              </div>
            </div>

            {/* Social Links */}
            <div className="mt-4 flex justify-center gap-2 md:justify-start">
              {personData.socialLinks.instagram && (
                <Button variant="outline" size="icon" asChild>
                  <a href={personData.socialLinks.instagram} target="_blank" rel="noopener noreferrer">
                    <Instagram className="h-4 w-4" />
                  </a>
                </Button>
              )}
              {personData.socialLinks.twitter && (
                <Button variant="outline" size="icon" asChild>
                  <a href={personData.socialLinks.twitter} target="_blank" rel="noopener noreferrer">
                    <Twitter className="h-4 w-4" />
                  </a>
                </Button>
              )}
              {personData.socialLinks.website && (
                <Button variant="outline" size="icon" asChild>
                  <a href={personData.socialLinks.website} target="_blank" rel="noopener noreferrer">
                    <Globe className="h-4 w-4" />
                  </a>
                </Button>
              )}
            </div>
          </div>
        </div>

        {/* Tabs Content */}
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList className="mb-6 w-full justify-start">
            <TabsTrigger value="filmography">Filmography</TabsTrigger>
            <TabsTrigger value="biography">Biography</TabsTrigger>
            <TabsTrigger value="photos">Photos</TabsTrigger>
          </TabsList>

          {/* Filmography Tab */}
          <TabsContent value="filmography" className="space-y-8">
            {/* As Director */}
            {filmography.director.length > 0 && (
              <section>
                <h3 className="mb-4 flex items-center gap-2 text-lg font-semibold">
                  <Film className="h-5 w-5 text-primary" />
                  As Director ({filmography.director.length})
                </h3>
                <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                  {filmography.director.map((film) => (
                    <FilmCard
                      key={`dir-${film.id}`}
                      film={film}
                      onClick={() => onSelectMedia(film.id, film.type as "movie" | "series")}
                    />
                  ))}
                </div>
              </section>
            )}

            {/* As Writer */}
            {filmography.writer.length > 0 && (
              <section>
                <h3 className="mb-4 flex items-center gap-2 text-lg font-semibold">
                  <Film className="h-5 w-5 text-muted-foreground" />
                  As Writer ({filmography.writer.length})
                </h3>
                <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                  {filmography.writer.map((film) => (
                    <FilmCard
                      key={`wri-${film.id}`}
                      film={film}
                      onClick={() => onSelectMedia(film.id, film.type as "movie" | "series")}
                    />
                  ))}
                </div>
              </section>
            )}

            {/* As Producer */}
            {filmography.producer.length > 0 && (
              <section>
                <h3 className="mb-4 flex items-center gap-2 text-lg font-semibold">
                  <Film className="h-5 w-5 text-muted-foreground" />
                  As Producer ({filmography.producer.length})
                </h3>
                <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                  {filmography.producer.map((film) => (
                    <FilmCard
                      key={`pro-${film.id}`}
                      film={film}
                      onClick={() => onSelectMedia(film.id, film.type as "movie" | "series")}
                    />
                  ))}
                </div>
              </section>
            )}
          </TabsContent>

          {/* Biography Tab */}
          <TabsContent value="biography">
            <div className="prose prose-invert max-w-none">
              {personData.biography.split('\n\n').map((paragraph, index) => (
                <p key={index} className="mb-4 text-muted-foreground leading-relaxed">
                  {paragraph}
                </p>
              ))}
            </div>
          </TabsContent>

          {/* Photos Tab */}
          <TabsContent value="photos">
            <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
              {personImages.map((image) => (
                <div
                  key={image.id}
                  className="group relative aspect-[2/3] cursor-pointer overflow-hidden rounded-lg bg-muted"
                >
                  <img
                    src={resolveArtwork(image.url)}
                    alt=""
                    className="h-full w-full object-cover transition-transform group-hover:scale-105"
                  />
                </div>
              ))}
            </div>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  )
}

// Film Card Component
function FilmCard({
  film,
  onClick
}: {
  film: { id: string; title: string; year: number; rating: number; poster: string; role: string }
  onClick: () => void
}) {
  return (
    <button
      onClick={onClick}
      className="group text-left"
    >
      <div className="relative aspect-[2/3] overflow-hidden rounded-lg bg-muted transition-all group-hover:ring-2 group-hover:ring-primary">
        <img
          src={resolveArtwork(film.poster)}
          alt={film.title}
          className="h-full w-full object-cover transition-transform group-hover:scale-105"
        />
        <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 transition-opacity group-hover:opacity-100" />
        <div className="absolute bottom-0 left-0 right-0 p-2 opacity-0 transition-opacity group-hover:opacity-100">
          <div className="flex items-center justify-center">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary text-primary-foreground">
              <Play className="h-4 w-4" />
            </div>
          </div>
        </div>
      </div>
      <h4 className="mt-2 truncate text-sm font-medium">{film.title}</h4>
      <div className="flex items-center gap-2 text-xs text-muted-foreground">
        <span>{film.year}</span>
        <span>·</span>
        <div className="flex items-center gap-0.5">
          <Star className="h-3 w-3 fill-accent text-accent" />
          <span>{film.rating}</span>
        </div>
      </div>
      <p className="mt-0.5 truncate text-xs text-muted-foreground/70">{film.role}</p>
    </button>
  )
}
