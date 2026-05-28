/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  transpilePackages: ['@nako/sdk'],
  experimental: {
    externalDir: true,
  },
  images: {
    unoptimized: true,
  },
}

export default nextConfig
