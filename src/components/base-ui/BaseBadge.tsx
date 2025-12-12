import React from 'react';
import { cn } from '@/lib/utils.ts';

export interface BaseBadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning';
  size?: 'sm' | 'default' | 'lg';
  children: React.ReactNode;
}

/**
 * BaseUI: BaseBadge
 * 基础徽章组件
 */
const BaseBadge = React.forwardRef<HTMLDivElement, BaseBadgeProps>(
  (
    {
      className,
      variant = 'default',
      size = 'default',
      children,
      ...props
    },
    ref
  ) => {
    const variantClasses = {
      default: 'bg-blue-600 text-white',
      secondary: 'bg-gray-600 text-gray-100',
      destructive: 'bg-red-600 text-white',
      outline: 'border border-gray-300 text-gray-700',
      success: 'bg-green-600 text-white',
      warning: 'bg-yellow-600 text-white',
    };

    const sizeClasses = {
      sm: 'px-2 py-0.5 text-xs',
      default: 'px-2.5 py-1 text-sm',
      lg: 'px-3 py-1.5 text-base',
    };

    return (
      <div
        ref={ref}
        className={cn(
          'inline-flex items-center rounded-full font-medium',
          variantClasses[variant],
          sizeClasses[size],
          className
        )}
        {...props}
      >
        {children}
      </div>
    );
  }
);

BaseBadge.displayName = 'BaseBadge';

export { BaseBadge };
