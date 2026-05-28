import type { Metadata, Viewport } from 'next'
import { QueryProvider } from '@/lib/query-provider'
import './globals.css'

export const metadata: Metadata = {
  title: 'Nako - 你的私人媒体库',
  description: '一个自托管的本地媒体系统，用于组织、管理和播放你的电影、剧集和个人收藏',
  generator: 'Nako',
}

export const viewport: Viewport = {
  themeColor: '#1a1a1f',
  colorScheme: 'dark',
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="zh-CN" className="bg-background" suppressHydrationWarning>
      <head>
        <script
          dangerouslySetInnerHTML={{
            __html: `
              // Suppress ResizeObserver loop error
              const resizeObserverErr = window.onerror;
              window.onerror = function(msg, url, line, col, error) {
                if (msg && msg.toString().includes('ResizeObserver loop')) {
                  return true;
                }
                return resizeObserverErr ? resizeObserverErr(msg, url, line, col, error) : false;
              };
            `,
          }}
        />
      </head>
      <body className="font-sans antialiased">
        <QueryProvider>
          {children}
        </QueryProvider>
      </body>
    </html>
  )
}
