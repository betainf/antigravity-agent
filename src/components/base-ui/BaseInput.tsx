import React from 'react';
import { cn } from '@/lib/utils.ts';
import { Eye, EyeOff, Lock } from 'lucide-react';

export interface BaseInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  containerClassName?: string;
}

export interface BasePasswordInputProps extends Omit<BaseInputProps, 'type' | 'rightIcon'> {
  showPasswordToggle?: boolean;
}

/**
 * BaseUI: BaseInput
 * 基础输入框组件
 * 支持标签、错误信息、左右图标
 */
const BaseInput = React.forwardRef<HTMLInputElement, BaseInputProps>(
  (
    {
      className,
      label,
      error,
      leftIcon,
      rightIcon,
      containerClassName,
      id,
      ...props
    },
    ref
  ) => {
    const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`;

    return (
      <div className={cn('space-y-2', containerClassName)}>
        {label && (
          <label
            htmlFor={inputId}
            className={cn(
              'text-sm font-medium',
              error ? 'text-red-600 dark:text-red-400' : 'text-gray-700 dark:text-gray-300'
            )}
          >
            {label}
          </label>
        )}

        <div className="relative">
          {leftIcon && (
            <div className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400">
              {leftIcon}
            </div>
          )}

          <input
            id={inputId}
            ref={ref}
            className={cn(
              // 基础样式
              'w-full px-4 py-3 rounded-lg',
              'bg-gray-50 dark:bg-gray-800',
              'border border-gray-300 dark:border-gray-600',
              'text-gray-900 dark:text-white',
              'placeholder-gray-500 dark:placeholder-gray-400',
              'focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent',
              'transition-all duration-200',

              // 尺寸调整
              leftIcon && 'pl-10',
              rightIcon && 'pr-10',

              // 错误状态
              error && 'border-red-500 focus:ring-red-500',

              className
            )}
            {...props}
          />

          {rightIcon && (
            <div className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400">
              {rightIcon}
            </div>
          )}
        </div>

        {error && (
          <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
        )}
      </div>
    );
  }
);

BaseInput.displayName = 'BaseInput';

/**
 * BaseUI: BasePasswordInput
 * 密码输入框组件，支持密码可见性切换
 */
const BasePasswordInput = React.forwardRef<HTMLInputElement, BasePasswordInputProps>(
  (
    {
      showPasswordToggle = true,
      ...props
    },
    ref
  ) => {
    const [showPassword, setShowPassword] = React.useState(false);

    return (
      <BaseInput
        ref={ref}
        type={showPassword ? 'text' : 'password'}
        leftIcon={<Lock className="h-4 w-4" />}
        rightIcon={
          showPasswordToggle ? (
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
            >
              {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </button>
          ) : undefined
        }
        {...props}
      />
    );
  }
);

BasePasswordInput.displayName = 'BasePasswordInput';

export { BaseInput, BasePasswordInput };
