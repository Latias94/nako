"use client"
import { resolveArtwork } from '@/lib/artwork'

import { useState } from "react"
import { Plus, Settings, Lock, Pencil, ChevronLeft } from "lucide-react"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"

// 用户数据
const users = [
  {
    id: "1",
    name: "Admin",
    avatar: "/avatars/avatar-1.jpg",
    isAdmin: true,
    hasPin: false
  },
  {
    id: "2",
    name: "家人",
    avatar: "/avatars/avatar-2.jpg",
    isAdmin: false,
    hasPin: true
  },
  {
    id: "3",
    name: "孩子",
    avatar: "/avatars/avatar-3.jpg",
    isAdmin: false,
    hasPin: false,
    isKids: true
  },
  {
    id: "4",
    name: "访客",
    avatar: "/avatars/avatar-4.jpg",
    isAdmin: false,
    hasPin: false
  },
]

interface UserSelectPageProps {
  onSelectUser: (userId: string) => void
  onManageProfiles?: () => void
  onBack?: () => void
}

export function UserSelectPage({ onSelectUser, onManageProfiles, onBack }: UserSelectPageProps) {
  const [isEditing, setIsEditing] = useState(false)
  const [pinInput, setPinInput] = useState("")
  const [selectedUser, setSelectedUser] = useState<string | null>(null)
  const [showPinModal, setShowPinModal] = useState(false)

  const handleUserClick = (user: typeof users[0]) => {
    if (isEditing) {
      // 编辑模式，打开编辑
      return
    }

    if (user.hasPin) {
      setSelectedUser(user.id)
      setShowPinModal(true)
      setPinInput("")
    } else {
      onSelectUser(user.id)
    }
  }

  const handlePinSubmit = () => {
    // 模拟 PIN 验证
    if (pinInput.length === 4) {
      onSelectUser(selectedUser!)
      setShowPinModal(false)
      setPinInput("")
    }
  }

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-background">
      {/* Back Button */}
      {onBack && (
        <Button
          variant="ghost"
          size="icon"
          onClick={onBack}
          className="absolute left-4 top-4"
        >
          <ChevronLeft className="h-5 w-5" />
        </Button>
      )}

      {/* Logo */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold tracking-tight text-primary">Nako</h1>
      </div>

      {/* 标题 */}
      <h2 className="mb-10 text-2xl font-medium text-foreground md:text-3xl">
        {isEditing ? "管理用户" : "谁在观看？"}
      </h2>

      {/* 用户网格 */}
      <div className="mb-8 flex flex-wrap justify-center gap-4 px-4 md:gap-6">
        {users.map((user) => (
          <button
            key={user.id}
            onClick={() => handleUserClick(user)}
            className="group flex flex-col items-center gap-3"
          >
            {/* 头像 */}
            <div className={cn(
              "relative h-24 w-24 overflow-hidden rounded-lg transition-all md:h-32 md:w-32",
              "ring-2 ring-transparent group-hover:ring-foreground",
              isEditing && "opacity-50"
            )}>
              <img
                src={resolveArtwork(user.avatar)}
                alt={user.name}
                className="h-full w-full object-cover"
              />

              {/* PIN 图标 */}
              {user.hasPin && !isEditing && (
                <div className="absolute bottom-1 right-1 flex h-5 w-5 items-center justify-center rounded-full bg-black/60">
                  <Lock className="h-3 w-3 text-white" />
                </div>
              )}

              {/* 管理员标记 */}
              {user.isAdmin && (
                <div className="absolute left-1 top-1 rounded bg-primary px-1.5 py-0.5 text-[10px] font-medium text-primary-foreground">
                  管理员
                </div>
              )}

              {/* 儿童模式标记 */}
              {user.isKids && (
                <div className="absolute left-1 top-1 rounded bg-cyan-500 px-1.5 py-0.5 text-[10px] font-medium text-white">
                  儿童
                </div>
              )}

              {/* 编辑覆盖层 */}
              {isEditing && (
                <div className="absolute inset-0 flex items-center justify-center bg-black/60">
                  <Pencil className="h-6 w-6 text-white" />
                </div>
              )}
            </div>

            {/* 用户名 */}
            <span className={cn(
              "text-sm text-muted-foreground transition-colors group-hover:text-foreground",
              "md:text-base"
            )}>
              {user.name}
            </span>
          </button>
        ))}

        {/* 添加用户按钮 */}
        {isEditing && (
          <button
            onClick={onManageProfiles}
            className="group flex flex-col items-center gap-3"
          >
            <div className={cn(
              "flex h-24 w-24 items-center justify-center rounded-lg border-2 border-dashed border-muted-foreground/30 transition-all md:h-32 md:w-32",
              "group-hover:border-foreground group-hover:bg-secondary"
            )}>
              <Plus className="h-10 w-10 text-muted-foreground transition-colors group-hover:text-foreground" />
            </div>
            <span className="text-sm text-muted-foreground group-hover:text-foreground md:text-base">
              添加用户
            </span>
          </button>
        )}
      </div>

      {/* 操作按钮 */}
      <div className="flex gap-4">
        <Button
          variant={isEditing ? "default" : "outline"}
          size="lg"
          onClick={() => setIsEditing(!isEditing)}
          className="gap-2"
        >
          {isEditing ? (
            "完成"
          ) : (
            <>
              <Settings className="h-4 w-4" />
              管理用户
            </>
          )}
        </Button>
      </div>

      {/* PIN 输入模态框 */}
      {showPinModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80">
          <div className="w-full max-w-sm rounded-lg bg-card p-6 shadow-2xl">
            <h3 className="mb-2 text-center text-lg font-medium">输入 PIN 码</h3>
            <p className="mb-6 text-center text-sm text-muted-foreground">
              此用户需要 PIN 码才能访问
            </p>

            {/* PIN 输入 */}
            <div className="mb-6 flex justify-center gap-3">
              {[0, 1, 2, 3].map((i) => (
                <div
                  key={i}
                  className={cn(
                    "flex h-12 w-12 items-center justify-center rounded-lg border-2 text-xl font-bold",
                    pinInput.length > i
                      ? "border-primary bg-primary/10 text-primary"
                      : "border-border bg-secondary"
                  )}
                >
                  {pinInput[i] ? "●" : ""}
                </div>
              ))}
            </div>

            {/* 数字键盘 */}
            <div className="mb-4 grid grid-cols-3 gap-2">
              {[1, 2, 3, 4, 5, 6, 7, 8, 9, null, 0, "del"].map((num, i) => (
                <button
                  key={i}
                  onClick={() => {
                    if (num === "del") {
                      setPinInput(prev => prev.slice(0, -1))
                    } else if (num !== null && pinInput.length < 4) {
                      const newPin = pinInput + num
                      setPinInput(newPin)
                      if (newPin.length === 4) {
                        setTimeout(() => handlePinSubmit(), 200)
                      }
                    }
                  }}
                  disabled={num === null}
                  className={cn(
                    "flex h-12 items-center justify-center rounded-lg text-lg font-medium transition-colors",
                    num === null
                      ? "invisible"
                      : "bg-secondary hover:bg-secondary/80 active:bg-primary/20"
                  )}
                >
                  {num === "del" ? "←" : num}
                </button>
              ))}
            </div>

            {/* 取消按钮 */}
            <Button
              variant="ghost"
              className="w-full"
              onClick={() => {
                setShowPinModal(false)
                setPinInput("")
              }}
            >
              取消
            </Button>
          </div>
        </div>
      )}

      {/* 服务器信息 */}
      <div className="absolute bottom-6 text-center">
        <p className="text-xs text-muted-foreground">
          连接到 <span className="text-foreground">Home Server</span>
        </p>
        <p className="mt-1 text-[10px] text-muted-foreground/60">
          192.168.1.100:8096
        </p>
      </div>
    </div>
  )
}
