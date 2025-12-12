import React from 'react';
import { cn } from '@/lib/utils.ts';

export interface BaseOverlayProps extends React.HTMLAttributes<HTMLDivElement> {
  blur?: boolean;
  opacity?: 'none' | 'sm' | 'default' | 'md' | 'lg';
}

/**
 * BaseUI: BaseOverlay
 * 基础遮罩层组件
 */
const BaseOverlay = React.forwardRef<HTMLDivElement, BaseOverlayProps>(
  (
    {
      className,
      blur = false,
      opacity = 'default',
      children,
      ...props
    },
    ref
  ) => {
    const opacityClasses = {
      none: 'bg-transparent',
      sm: 'bg-black/20',
      default: 'bg-black/50',
      md: 'bg-black/60',
      lg: 'bg-black/70',
    };

    const blurClass = blur ? 'backdrop-blur-sm' : '';

    return (
      <div
        ref={ref}
        className={cn(
          'fixed inset-0 z-50',
          opacityClasses[opacity],
          blurClass,
          className
        )}
        {...props}
      >
        {children}
      </div>
    );
  }
);

BaseOverlay.displayName = 'BaseOverlay';

export { BaseOverlay };
