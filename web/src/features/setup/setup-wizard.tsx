"use client"

import { useState } from "react"
import { 
  Server, FolderPlus, User, Check, ChevronRight, ChevronLeft, 
  Wifi, WifiOff, Loader2, Film, Tv, Music, Camera, FolderOpen,
  Eye, EyeOff, Shield, Globe, Sparkles
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent } from "@/components/ui/card"
import { Checkbox } from "@/components/ui/checkbox"
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group"
import { Progress } from "@/components/ui/progress"
import { cn } from "@/lib/utils"

interface SetupWizardProps {
  onComplete: () => void
}

type SetupStep = "welcome" | "server" | "libraries" | "account" | "preferences" | "complete"

const steps: { id: SetupStep; label: string; icon: React.ComponentType<{ className?: string }> }[] = [
  { id: "welcome", label: "Welcome", icon: Sparkles },
  { id: "server", label: "Server", icon: Server },
  { id: "libraries", label: "Libraries", icon: FolderPlus },
  { id: "account", label: "Account", icon: User },
  { id: "preferences", label: "Preferences", icon: Shield },
  { id: "complete", label: "Complete", icon: Check },
]

const libraryTypes = [
  { id: "movies", label: "Movies", icon: Film, description: "Feature films, documentaries" },
  { id: "tvshows", label: "TV Shows", icon: Tv, description: "Series, episodes, seasons" },
  { id: "music", label: "Music", icon: Music, description: "Albums, artists, playlists" },
  { id: "photos", label: "Photos", icon: Camera, description: "Photo albums, galleries" },
]

export function SetupWizard({ onComplete }: SetupWizardProps) {
  const [currentStep, setCurrentStep] = useState<SetupStep>("welcome")
  const [isConnecting, setIsConnecting] = useState(false)
  const [connectionStatus, setConnectionStatus] = useState<"idle" | "success" | "error">("idle")
  
  // Form states
  const [serverUrl, setServerUrl] = useState("")
  const [serverPort, setServerPort] = useState("8096")
  const [libraries, setLibraries] = useState<{ type: string; path: string; name: string }[]>([])
  const [selectedLibraryType, setSelectedLibraryType] = useState("")
  const [libraryPath, setLibraryPath] = useState("")
  const [libraryName, setLibraryName] = useState("")
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
  const [showPassword, setShowPassword] = useState(false)
  const [enableRemoteAccess, setEnableRemoteAccess] = useState(true)
  const [enableAutoScan, setEnableAutoScan] = useState(true)
  const [metadataLanguage, setMetadataLanguage] = useState("zh-CN")

  const currentStepIndex = steps.findIndex((s) => s.id === currentStep)
  const progress = ((currentStepIndex) / (steps.length - 1)) * 100

  const handleTestConnection = async () => {
    setIsConnecting(true)
    // Simulate connection test
    await new Promise((resolve) => setTimeout(resolve, 2000))
    setConnectionStatus(serverUrl ? "success" : "error")
    setIsConnecting(false)
  }

  const handleAddLibrary = () => {
    if (selectedLibraryType && libraryPath) {
      setLibraries([
        ...libraries,
        {
          type: selectedLibraryType,
          path: libraryPath,
          name: libraryName || `My ${selectedLibraryType}`,
        },
      ])
      setSelectedLibraryType("")
      setLibraryPath("")
      setLibraryName("")
    }
  }

  const handleRemoveLibrary = (index: number) => {
    setLibraries(libraries.filter((_, i) => i !== index))
  }

  const goNext = () => {
    const nextIndex = currentStepIndex + 1
    if (nextIndex < steps.length) {
      setCurrentStep(steps[nextIndex].id)
    }
  }

  const goPrevious = () => {
    const prevIndex = currentStepIndex - 1
    if (prevIndex >= 0) {
      setCurrentStep(steps[prevIndex].id)
    }
  }

  const canProceed = () => {
    switch (currentStep) {
      case "welcome":
        return true
      case "server":
        return connectionStatus === "success"
      case "libraries":
        return libraries.length > 0
      case "account":
        return username.length >= 3 && password.length >= 6
      case "preferences":
        return true
      default:
        return true
    }
  }

  return (
    <div className="flex min-h-screen flex-col bg-background">
      {/* Progress Header */}
      {currentStep !== "welcome" && currentStep !== "complete" && (
        <header className="border-b border-border/50 bg-background/95 backdrop-blur">
          <div className="mx-auto max-w-3xl px-4 py-4">
            <div className="mb-3 flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Setup Progress</span>
              <span className="font-medium">{Math.round(progress)}%</span>
            </div>
            <Progress value={progress} className="h-2" />
            
            {/* Step Indicators */}
            <div className="mt-4 flex items-center justify-between">
              {steps.slice(1, -1).map((step, index) => {
                const stepIndex = index + 1
                const isActive = currentStepIndex === stepIndex
                const isCompleted = currentStepIndex > stepIndex
                
                return (
                  <div
                    key={step.id}
                    className={cn(
                      "flex items-center gap-2 text-sm",
                      isActive && "text-primary",
                      isCompleted && "text-muted-foreground",
                      !isActive && !isCompleted && "text-muted-foreground/50"
                    )}
                  >
                    <div
                      className={cn(
                        "flex h-8 w-8 items-center justify-center rounded-full border-2 transition-colors",
                        isActive && "border-primary bg-primary text-primary-foreground",
                        isCompleted && "border-primary bg-primary/20 text-primary",
                        !isActive && !isCompleted && "border-muted-foreground/30"
                      )}
                    >
                      {isCompleted ? (
                        <Check className="h-4 w-4" />
                      ) : (
                        <step.icon className="h-4 w-4" />
                      )}
                    </div>
                    <span className="hidden sm:inline">{step.label}</span>
                  </div>
                )
              })}
            </div>
          </div>
        </header>
      )}

      {/* Main Content */}
      <main className="flex flex-1 items-center justify-center p-4">
        <div className="w-full max-w-2xl">
          {/* Welcome Step */}
          {currentStep === "welcome" && (
            <div className="text-center">
              <div className="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-full bg-primary/10">
                <Sparkles className="h-10 w-10 text-primary" />
              </div>
              <h1 className="mb-3 text-3xl font-bold">Welcome to Nako</h1>
              <p className="mb-8 text-lg text-muted-foreground">
                Your personal media server. Let&apos;s get you set up in just a few steps.
              </p>
              <div className="space-y-4">
                <Card className="border-border/50 bg-card/50">
                  <CardContent className="flex items-center gap-4 p-4">
                    <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
                      <Server className="h-6 w-6 text-primary" />
                    </div>
                    <div className="text-left">
                      <h3 className="font-medium">Connect to Server</h3>
                      <p className="text-sm text-muted-foreground">Set up your media server connection</p>
                    </div>
                  </CardContent>
                </Card>
                <Card className="border-border/50 bg-card/50">
                  <CardContent className="flex items-center gap-4 p-4">
                    <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
                      <FolderPlus className="h-6 w-6 text-primary" />
                    </div>
                    <div className="text-left">
                      <h3 className="font-medium">Add Media Libraries</h3>
                      <p className="text-sm text-muted-foreground">Tell us where your media files are</p>
                    </div>
                  </CardContent>
                </Card>
                <Card className="border-border/50 bg-card/50">
                  <CardContent className="flex items-center gap-4 p-4">
                    <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
                      <User className="h-6 w-6 text-primary" />
                    </div>
                    <div className="text-left">
                      <h3 className="font-medium">Create Admin Account</h3>
                      <p className="text-sm text-muted-foreground">Set up your administrator account</p>
                    </div>
                  </CardContent>
                </Card>
              </div>
              <Button size="lg" className="mt-8" onClick={goNext}>
                Get Started <ChevronRight className="ml-2 h-4 w-4" />
              </Button>
            </div>
          )}

          {/* Server Connection Step */}
          {currentStep === "server" && (
            <div>
              <h2 className="mb-2 text-2xl font-bold">Server Connection</h2>
              <p className="mb-6 text-muted-foreground">
                Enter your media server details to establish a connection.
              </p>
              
              <div className="space-y-4">
                <div className="grid gap-4 sm:grid-cols-3">
                  <div className="sm:col-span-2">
                    <Label htmlFor="serverUrl">Server URL</Label>
                    <Input
                      id="serverUrl"
                      placeholder="http://localhost or IP address"
                      value={serverUrl}
                      onChange={(e) => {
                        setServerUrl(e.target.value)
                        setConnectionStatus("idle")
                      }}
                      className="mt-1.5"
                    />
                  </div>
                  <div>
                    <Label htmlFor="serverPort">Port</Label>
                    <Input
                      id="serverPort"
                      placeholder="8096"
                      value={serverPort}
                      onChange={(e) => {
                        setServerPort(e.target.value)
                        setConnectionStatus("idle")
                      }}
                      className="mt-1.5"
                    />
                  </div>
                </div>

                <Button
                  variant="outline"
                  onClick={handleTestConnection}
                  disabled={!serverUrl || isConnecting}
                  className="w-full"
                >
                  {isConnecting ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      Testing Connection...
                    </>
                  ) : connectionStatus === "success" ? (
                    <>
                      <Wifi className="mr-2 h-4 w-4 text-green-500" />
                      Connection Successful
                    </>
                  ) : connectionStatus === "error" ? (
                    <>
                      <WifiOff className="mr-2 h-4 w-4 text-destructive" />
                      Connection Failed - Try Again
                    </>
                  ) : (
                    <>
                      <Wifi className="mr-2 h-4 w-4" />
                      Test Connection
                    </>
                  )}
                </Button>

                {connectionStatus === "success" && (
                  <Card className="border-green-500/30 bg-green-500/10">
                    <CardContent className="flex items-center gap-3 p-4">
                      <Check className="h-5 w-5 text-green-500" />
                      <div>
                        <p className="font-medium text-green-500">Connected to Server</p>
                        <p className="text-sm text-muted-foreground">
                          {serverUrl}:{serverPort}
                        </p>
                      </div>
                    </CardContent>
                  </Card>
                )}
              </div>
            </div>
          )}

          {/* Libraries Step */}
          {currentStep === "libraries" && (
            <div>
              <h2 className="mb-2 text-2xl font-bold">Media Libraries</h2>
              <p className="mb-6 text-muted-foreground">
                Add folders containing your media files. You can add more libraries later.
              </p>

              {/* Library Type Selection */}
              <div className="mb-6 grid grid-cols-2 gap-3 sm:grid-cols-4">
                {libraryTypes.map((type) => (
                  <button
                    key={type.id}
                    onClick={() => setSelectedLibraryType(type.id)}
                    className={cn(
                      "flex flex-col items-center gap-2 rounded-lg border-2 p-4 transition-all",
                      selectedLibraryType === type.id
                        ? "border-primary bg-primary/10"
                        : "border-border/50 hover:border-primary/50"
                    )}
                  >
                    <type.icon className={cn(
                      "h-8 w-8",
                      selectedLibraryType === type.id ? "text-primary" : "text-muted-foreground"
                    )} />
                    <span className="text-sm font-medium">{type.label}</span>
                  </button>
                ))}
              </div>

              {/* Add Library Form */}
              {selectedLibraryType && (
                <Card className="mb-6 border-border/50 bg-card/50">
                  <CardContent className="space-y-4 p-4">
                    <div>
                      <Label htmlFor="libraryName">Library Name</Label>
                      <Input
                        id="libraryName"
                        placeholder={`My ${selectedLibraryType}`}
                        value={libraryName}
                        onChange={(e) => setLibraryName(e.target.value)}
                        className="mt-1.5"
                      />
                    </div>
                    <div>
                      <Label htmlFor="libraryPath">Folder Path</Label>
                      <div className="mt-1.5 flex gap-2">
                        <Input
                          id="libraryPath"
                          placeholder="/media/movies"
                          value={libraryPath}
                          onChange={(e) => setLibraryPath(e.target.value)}
                        />
                        <Button variant="outline" size="icon">
                          <FolderOpen className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>
                    <Button onClick={handleAddLibrary} disabled={!libraryPath}>
                      Add Library
                    </Button>
                  </CardContent>
                </Card>
              )}

              {/* Added Libraries */}
              {libraries.length > 0 && (
                <div className="space-y-2">
                  <Label>Added Libraries ({libraries.length})</Label>
                  {libraries.map((lib, index) => {
                    const typeInfo = libraryTypes.find((t) => t.id === lib.type)
                    const TypeIcon = typeInfo?.icon || FolderPlus
                    return (
                      <Card key={index} className="border-border/50">
                        <CardContent className="flex items-center justify-between p-3">
                          <div className="flex items-center gap-3">
                            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-secondary">
                              <TypeIcon className="h-5 w-5 text-muted-foreground" />
                            </div>
                            <div>
                              <p className="font-medium">{lib.name}</p>
                              <p className="text-sm text-muted-foreground">{lib.path}</p>
                            </div>
                          </div>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleRemoveLibrary(index)}
                            className="text-destructive hover:text-destructive"
                          >
                            Remove
                          </Button>
                        </CardContent>
                      </Card>
                    )
                  })}
                </div>
              )}
            </div>
          )}

          {/* Account Step */}
          {currentStep === "account" && (
            <div>
              <h2 className="mb-2 text-2xl font-bold">Create Admin Account</h2>
              <p className="mb-6 text-muted-foreground">
                Set up your administrator account to manage the server.
              </p>

              <div className="space-y-4">
                <div>
                  <Label htmlFor="username">Username</Label>
                  <Input
                    id="username"
                    placeholder="admin"
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    className="mt-1.5"
                  />
                  {username.length > 0 && username.length < 3 && (
                    <p className="mt-1 text-sm text-destructive">Username must be at least 3 characters</p>
                  )}
                </div>
                <div>
                  <Label htmlFor="password">Password</Label>
                  <div className="relative mt-1.5">
                    <Input
                      id="password"
                      type={showPassword ? "text" : "password"}
                      placeholder="Enter a secure password"
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      className="pr-10"
                    />
                    <button
                      type="button"
                      onClick={() => setShowPassword(!showPassword)}
                      className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                    >
                      {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                    </button>
                  </div>
                  {password.length > 0 && password.length < 6 && (
                    <p className="mt-1 text-sm text-destructive">Password must be at least 6 characters</p>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Preferences Step */}
          {currentStep === "preferences" && (
            <div>
              <h2 className="mb-2 text-2xl font-bold">Preferences</h2>
              <p className="mb-6 text-muted-foreground">
                Configure your server preferences. You can change these later in settings.
              </p>

              <div className="space-y-6">
                <div>
                  <Label className="text-base">Metadata Language</Label>
                  <RadioGroup
                    value={metadataLanguage}
                    onValueChange={setMetadataLanguage}
                    className="mt-3 space-y-2"
                  >
                    <div className="flex items-center space-x-3 rounded-lg border border-border/50 p-3">
                      <RadioGroupItem value="zh-CN" id="zh-CN" />
                      <Label htmlFor="zh-CN" className="flex-1 cursor-pointer">
                        <div className="font-medium">Simplified Chinese</div>
                        <div className="text-sm text-muted-foreground">Fetch metadata in Simplified Chinese</div>
                      </Label>
                    </div>
                    <div className="flex items-center space-x-3 rounded-lg border border-border/50 p-3">
                      <RadioGroupItem value="en-US" id="en-US" />
                      <Label htmlFor="en-US" className="flex-1 cursor-pointer">
                        <div className="font-medium">English</div>
                        <div className="text-sm text-muted-foreground">Fetch metadata in English</div>
                      </Label>
                    </div>
                  </RadioGroup>
                </div>

                <div className="space-y-4">
                  <div className="flex items-center justify-between rounded-lg border border-border/50 p-4">
                    <div className="flex items-center gap-3">
                      <Globe className="h-5 w-5 text-muted-foreground" />
                      <div>
                        <div className="font-medium">Remote Access</div>
                        <div className="text-sm text-muted-foreground">Allow access from outside your network</div>
                      </div>
                    </div>
                    <Checkbox
                      checked={enableRemoteAccess}
                      onCheckedChange={(checked) => setEnableRemoteAccess(checked as boolean)}
                    />
                  </div>
                  <div className="flex items-center justify-between rounded-lg border border-border/50 p-4">
                    <div className="flex items-center gap-3">
                      <FolderPlus className="h-5 w-5 text-muted-foreground" />
                      <div>
                        <div className="font-medium">Auto-scan Libraries</div>
                        <div className="text-sm text-muted-foreground">Automatically scan for new media</div>
                      </div>
                    </div>
                    <Checkbox
                      checked={enableAutoScan}
                      onCheckedChange={(checked) => setEnableAutoScan(checked as boolean)}
                    />
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Complete Step */}
          {currentStep === "complete" && (
            <div className="text-center">
              <div className="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-full bg-green-500/10">
                <Check className="h-10 w-10 text-green-500" />
              </div>
              <h1 className="mb-3 text-3xl font-bold">Setup Complete!</h1>
              <p className="mb-8 text-lg text-muted-foreground">
                Your media server is ready to use. Start exploring your library!
              </p>
              
              <div className="mb-8 rounded-lg border border-border/50 bg-card/50 p-6 text-left">
                <h3 className="mb-4 font-semibold">Summary</h3>
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Server</span>
                    <span>{serverUrl}:{serverPort}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Libraries</span>
                    <span>{libraries.length} libraries added</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Admin User</span>
                    <span>{username}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Remote Access</span>
                    <span>{enableRemoteAccess ? "Enabled" : "Disabled"}</span>
                  </div>
                </div>
              </div>

              <Button size="lg" onClick={onComplete}>
                Start Watching <ChevronRight className="ml-2 h-4 w-4" />
              </Button>
            </div>
          )}

          {/* Navigation Buttons */}
          {currentStep !== "welcome" && currentStep !== "complete" && (
            <div className="mt-8 flex justify-between">
              <Button variant="outline" onClick={goPrevious}>
                <ChevronLeft className="mr-2 h-4 w-4" />
                Back
              </Button>
              <Button onClick={goNext} disabled={!canProceed()}>
                {currentStep === "preferences" ? "Complete Setup" : "Continue"}
                <ChevronRight className="ml-2 h-4 w-4" />
              </Button>
            </div>
          )}
        </div>
      </main>
    </div>
  )
}
