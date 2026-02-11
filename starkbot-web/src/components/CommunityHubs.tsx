import { ExternalLink } from 'lucide-react'
import hubs from '../config/community-hubs.json'

export function CommunityHubs() {
  return (
    <section className="py-8 px-6">
      <div className="max-w-5xl mx-auto">
        <h2 className="text-2xl font-bold text-white mb-4">Community Hubs</h2>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
          {hubs.map((hub) => (
            <a
              key={hub.url}
              href={hub.url}
              target="_blank"
              rel="noopener noreferrer"
              className="group bg-gradient-to-br from-white/5 to-white/[0.02] rounded-2xl border border-white/10 hover:border-white/20 p-5 transition-all duration-300 hover:scale-[1.02]"
            >
              <div className="flex items-center justify-between mb-2">
                <h3 className="text-lg font-semibold text-white">{hub.title}</h3>
                <ExternalLink className="w-4 h-4 text-white/40 group-hover:text-white/70 transition-colors" />
              </div>
              <p className="text-sm text-white/50 leading-relaxed">{hub.description}</p>
            </a>
          ))}
        </div>
      </div>
    </section>
  )
}
