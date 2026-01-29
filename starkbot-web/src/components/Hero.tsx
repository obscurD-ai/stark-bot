import { Github, ChevronDown, Monitor, BookOpen, Wallet } from 'lucide-react'
import { Link } from 'react-router-dom'

export function Hero() {
  return (
    <section className="pt-32 pb-20 px-6">
      <div className="max-w-4xl mx-auto text-center">
        {/* Mascot/Logo */}
        <div className="mb-8 animate-float">
          <div className="w-32 h-32 mx-auto bg-gradient-to-br from-stark-400 via-stark-500 to-stark-600 rounded-3xl flex items-center justify-center glow transform rotate-3 hover:rotate-0 transition-transform duration-500">
            <div className="relative">
              <Monitor className="w-16 h-16 text-white" strokeWidth={1.5} />
              <div className="absolute top-6 left-4 flex gap-4">
                <div className="w-2 h-2 bg-white rounded-full" />
                <div className="w-2 h-2 bg-white rounded-full" />
              </div>
            </div>
          </div>
        </div>

        {/* Title */}
        <h1 className="text-5xl sm:text-7xl font-black mb-6 tracking-tight">
          <span className="gradient-text">StarkBot</span>
        </h1>

        {/* Tagline */}
        <p className="text-stark-400 text-xl sm:text-2xl font-semibold uppercase tracking-widest mb-2">
          Web3-Native AI Agent
        </p>

        {/* Badges */}
        <div className="flex flex-wrap gap-3 justify-center mb-6">
          <span className="px-3 py-1 bg-stark-500/20 border border-stark-500/50 rounded-full text-stark-400 text-sm font-medium">
            Sign In With Ethereum
          </span>
          <span className="px-3 py-1 bg-purple-500/20 border border-purple-500/50 rounded-full text-purple-400 text-sm font-medium">
            x402 Payments
          </span>
          <span className="px-3 py-1 bg-green-500/20 border border-green-500/50 rounded-full text-green-400 text-sm font-medium">
            DeFi Relay
          </span>
        </div>

        {/* Description */}
        <p className="text-slate-400 text-lg sm:text-xl max-w-2xl mx-auto leading-relaxed mb-12">
          A crypto-native AI assistant with wallet authentication, x402 micropayments, and autonomous agent capabilities.
          Open source, self-hostable, and built for the decentralized web.
        </p>

        {/* CTA Buttons */}
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <a
            href="https://github.com/ethereumdegen/stark-bot"
            target="_blank"
            rel="noopener noreferrer"
            className="px-8 py-4 bg-gradient-to-r from-stark-500 to-stark-600 hover:from-stark-400 hover:to-stark-500 text-white font-semibold rounded-xl transition-all duration-300 transform hover:scale-105 shadow-lg hover:shadow-stark-500/25 flex items-center justify-center gap-3"
          >
            <Github className="w-6 h-6" />
            View on GitHub
          </a>
          <Link
            to="/docs"
            className="px-8 py-4 bg-slate-800 hover:bg-slate-700 text-white font-semibold rounded-xl transition-all duration-300 border border-slate-700 hover:border-stark-500 flex items-center justify-center gap-3"
          >
            <BookOpen className="w-6 h-6" />
            Read the Docs
          </Link>
          <a
            href="#features"
            className="px-8 py-4 bg-slate-800 hover:bg-slate-700 text-white font-semibold rounded-xl transition-all duration-300 border border-slate-700 hover:border-stark-500 flex items-center justify-center gap-2"
          >
            Learn More
            <ChevronDown className="w-5 h-5" />
          </a>
        </div>
      </div>
    </section>
  )
}
