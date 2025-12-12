import React from 'react';
import { motion, useReducedMotion } from 'framer-motion';
import { cn } from '@/lib/utils.ts';
import { Loader2 } from 'lucide-react';

type NativeButtonProps = Omit<
  React.ButtonHTMLAttributes<HTMLButtonElement>,
  | 'onDrag'
  | 'onDragStart'
  | 'onDragEnd'
  | 'onAnimationStart'
  | 'onAnimationEnd'
  | 'onAnimationIteration'
>;

export interface BaseButtonProps extends NativeButtonProps {
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
  size?: 'default' | 'sm' | 'lg' | 'icon';
  isLoading?: boolean;
  loadingText?: string;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  fullWidth?: boolean;
}

// Vercel / Geist-ish 风格（高对比、干净、克制）
const buttonVariants: Record<NonNullable<BaseButtonProps['variant']>, string> = {
  default: cn(
    'bg-zinc-900 text-zinc-50 border border-zinc-900/90 shadow-sm shadow-black/20',
    'hover:bg-zinc-800 hover:border-zinc-800',
    'dark:bg-zinc-50 dark:text-zinc-900 dark:border-zinc-50 dark:hover:bg-zinc-200 dark:shadow-none'
  ),
  destructive: cn(
    'bg-red-600 text-white border border-red-600 shadow-sm shadow-red-600/30',
    'hover:bg-red-700 hover:border-red-700',
    'dark:bg-red-500 dark:border-red-500 dark:hover:bg-red-600 dark:hover:border-red-600'
  ),
  outline: cn(
    'bg-white text-zinc-900 border border-zinc-200 shadow-sm',
    'hover:bg-zinc-50 hover:border-zinc-300',
    'dark:bg-zinc-950 dark:text-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-900'
  ),
  secondary: cn(
    'bg-zinc-100 text-zinc-900 border border-transparent shadow-sm',
    'hover:bg-zinc-200',
    'dark:bg-zinc-800 dark:text-zinc-50 dark:hover:bg-zinc-700'
  ),
  ghost: cn(
    'bg-transparent text-zinc-900 border border-transparent',
    'hover:bg-zinc-100/70',
    'dark:text-zinc-50 dark:hover:bg-zinc-800/60'
  ),
  link: cn(
    'bg-transparent text-zinc-900 p-0 h-auto underline-offset-4',
    'hover:underline',
    'dark:text-zinc-50'
  ),
};

const buttonSizes: Record<NonNullable<BaseButtonProps['size']>, string> = {
  default: 'h-9 px-4 py-2 text-sm',
  sm: 'h-8 px-3 text-xs',
  lg: 'h-10 px-6 text-sm',
  icon: 'h-9 w-9 p-0',
};

/**
 * BaseUI: BaseButton
 * 纯UI按钮组件，不包含业务逻辑
 * 支持加载状态、图标、多种样式变体
 * 使用 framer-motion 提供轻量交互动画
 */
const BaseButton = React.forwardRef<HTMLButtonElement, BaseButtonProps>(
  (
    {
      className,
      variant = 'default',
      size = 'default',
      isLoading = false,
      loadingText,
      leftIcon,
      rightIcon,
      fullWidth = false,
      disabled,
      children,
      ...props
    },
    ref
  ) => {
    const isDisabled = disabled || isLoading;
    const shouldReduceMotion = useReducedMotion();
    const isLinkVariant = variant === 'link';
    const isGhostVariant = variant === 'ghost';
    const canHoverRing = !isLinkVariant;
    const canHoverShadow = !isLinkVariant && !isGhostVariant;

    const whileHover = shouldReduceMotion || isDisabled ? undefined : { y: -1, scale: 1.02 };
    const whileTap = shouldReduceMotion || isDisabled ? undefined : { y: 0, scale: 0.98 };

    return (
      <motion.button
        className={cn(
          // 基础样式
          'relative inline-flex items-center justify-center gap-2 whitespace-nowrap select-none cursor-pointer',
          'rounded-md font-medium leading-none',
          'transition-colors duration-150',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-zinc-400/60 focus-visible:ring-offset-2',
          'focus-visible:ring-offset-white dark:focus-visible:ring-offset-zinc-950',
          'disabled:pointer-events-none disabled:opacity-50 disabled:cursor-not-allowed',
          '[&_svg]:h-4 [&_svg]:w-4 [&_svg]:shrink-0',
          // Hover 反馈（link/ghost 更克制）
          canHoverRing && 'hover:ring-1 hover:ring-zinc-900/10 dark:hover:ring-white/10',
          canHoverShadow && 'hover:shadow-md',

          // 变体样式
          buttonVariants[variant],

          // 尺寸样式
          buttonSizes[size],

          // 加载状态
          isLoading && 'cursor-wait',

          // 宽度
          fullWidth && 'w-full',

          // 自定义类名
          className
        )}
        disabled={isDisabled}
        ref={ref}
        whileHover={whileHover}
        whileTap={whileTap}
        transition={{ type: 'spring', stiffness: 500, damping: 32 }}
        {...props}
      >
        {isLoading ? (
          <>
            <Loader2 className="h-4 w-4 animate-spin" />
            <span>{loadingText || '处理中...'}</span>
          </>
        ) : (
          <>
            {leftIcon && <span className="flex items-center">{leftIcon}</span>}
            {children && <span>{children}</span>}
            {rightIcon && <span className="flex items-center">{rightIcon}</span>}
          </>
        )}
      </motion.button>
    );
  }
);

BaseButton.displayName = 'BaseButton';

export { BaseButton };
