import React from 'react';
import { cn } from '@/lib/utils.ts';

export interface BaseAvatarProps extends React.HTMLAttributes<HTMLDivElement> {
  src?: string;
  alt?: string;
  size?: number;
}

/**
 * BaseUI: BaseAvatar
 * Aceternity 风格的基础头像组件，默认 48x48，支持图片与炫光渐变占位。
 */
const BaseAvatar = React.forwardRef<HTMLDivElement, BaseAvatarProps>(
  ({ className, src, alt = 'avatar', size = 48, style, ...props }, ref) => {
    const [hasError, setHasError] = React.useState(false);
    const dimension = size || 48;
    const showImage = src && !hasError;

    return (
      <div
        ref={ref}
        style={{ width: dimension, height: dimension, ...style }}
        className={cn(
          'relative inline-flex items-center justify-center overflow-hidden rounded-full',
          'ring-1 ring-white/15 shadow-[0_18px_60px_-28px_rgba(56,189,248,0.8)]',
          'bg-slate-900/40 backdrop-blur supports-[backdrop-filter]:backdrop-blur',
          className
        )}
        {...props}
      >
        <div
          aria-hidden
          className="absolute inset-0 bg-gradient-to-br from-[#0f172a] via-[#0ea5e9]/50 to-[#22d3ee]/60"
        />
        <div
          aria-hidden
          className="absolute inset-[-20%] opacity-60 blur-xl bg-[radial-gradient(circle_at_30%_20%,rgba(59,130,246,0.55),transparent_38%),radial-gradient(circle_at_80%_0%,rgba(236,72,153,0.45),transparent_34%),radial-gradient(circle_at_50%_80%,rgba(52,211,153,0.45),transparent_38%)]"
        />

        {showImage ? (
          <img
            src={src}
            alt={alt}
            className="absolute inset-0 h-full w-full object-cover"
            onError={() => setHasError(true)}
            loading="lazy"
          />
        ) : (
          <div className="relative z-10 flex h-full w-full items-center justify-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 64 64"
              className="h-3/4 w-3/4 drop-shadow-[0_6px_16px_rgba(15,23,42,0.45)]"
            >
              <defs>
                <linearGradient id="avatarGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                  <stop offset="0%" stopColor="#38bdf8" />
                  <stop offset="50%" stopColor="#a855f7" />
                  <stop offset="100%" stopColor="#22d3ee" />
                </linearGradient>
              </defs>
              <circle cx="32" cy="24" r="14" fill="url(#avatarGradient)" opacity="0.95" />
              <path
                d="M16 52c2-8 8.5-12 16-12s14 4 16 12"
                fill="url(#avatarGradient)"
                opacity="0.55"
              />
              <circle cx="26" cy="22" r="2.5" fill="#0b1221" />
              <circle cx="38" cy="22" r="2.5" fill="#0b1221" />
              <path
                d="M26 30c2.5 2 9.5 2 12 0"
                stroke="#0b1221"
                strokeWidth="2"
                strokeLinecap="round"
                fill="none"
              />
            </svg>
          </div>
        )}

        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 ring-1 ring-white/10"
        />
      </div>
    );
  }
);

BaseAvatar.displayName = 'BaseAvatar';

export { BaseAvatar };
