import React from 'react';
import { cn } from '@/lib/utils.ts';

export interface BaseProgressProps {
  value: number;
  max?: number;
  className?: string;
  showLabel?: boolean;
  size?: 'sm' | 'default' | 'lg';
  variant?: 'default' | 'success' | 'warning' | 'danger';
}

/**
 * BaseUI: BaseProgress
 * 基础进度条组件
 */
const BaseProgress = React.forwardRef<HTMLDivElement, BaseProgressProps>(
  (
    {
      value,
      max = 100,
      className,
      showLabel = false,
      size = 'default',
      variant = 'default',
      ...props
    },
    ref
  ) => {
    const percentage = Math.min(Math.max((value / max) * 100, 0), 100);

    const sizeClasses = {
      sm: 'h-2',
      default: 'h-3',
      lg: 'h-4',
    };

    const variantClasses = {
      default: 'bg-blue-500',
      success: 'bg-green-500',
      warning: 'bg-yellow-500',
      danger: 'bg-red-500',
    };

    return (
      <div ref={ref} className={cn('w-full', className)} {...props}>
        <div
          className={cn(
            'w-full bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden',
            sizeClasses[size]
          )}
        >
          <div
            className={cn(
              'h-full transition-all duration-300 ease-out rounded-full',
              variantClasses[variant]
            )}
            style={{ width: `${percentage}%` }}
          />
        </div>
        {showLabel && (
          <div className="flex justify-between text-sm text-gray-600 dark:text-gray-400 mt-1">
            <span>{value}</span>
            <span>{max}</span>
          </div>
        )}
      </div>
    );
  }
);

BaseProgress.displayName = 'BaseProgress';

export { BaseProgress };
