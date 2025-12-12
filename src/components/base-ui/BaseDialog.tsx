import React from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { X } from 'lucide-react';
import { cn } from '@/lib/utils.ts';

export interface BaseDialogProps {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  children: React.ReactNode;
}

export interface BaseDialogContentProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
  className?: string;
}

export interface BaseDialogHeaderProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

export interface BaseDialogFooterProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

export interface BaseDialogTitleProps extends React.HTMLAttributes<HTMLHeadingElement> {
  children: React.ReactNode;
}

export interface BaseDialogDescriptionProps extends React.HTMLAttributes<HTMLParagraphElement> {
  children: React.ReactNode;
}

/**
 * BaseUI: BaseDialog
 * 基础对话框组件，基于 Radix UI Dialog 封装
 * 提供头部、描述、底部按钮区域
 */
const BaseDialog: React.FC<BaseDialogProps> = ({ open, onOpenChange, children }) => {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay
          className={cn(
            'fixed inset-0 bg-black/50',
            'data-[state=open]:animate-in data-[state=closed]:animate-out',
            'data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0',
            'z-50'
          )}
        />
        {children}
      </Dialog.Portal>
    </Dialog.Root>
  );
};

const BaseDialogContent = React.forwardRef<HTMLDivElement, BaseDialogContentProps>(
  ({ children, className, ...props }, ref) => (
    <Dialog.Content
      ref={ref}
      className={cn(
        'fixed left-[50%] top-[50%] translate-x-[-50%] translate-y-[-50%]',
        'w-full max-w-md bg-white dark:bg-gray-900 rounded-xl shadow-2xl',
        'data-[state=open]:animate-in data-[state=closed]:animate-out',
        'data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0',
        'data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95',
        'data-[state=closed]:slide-out-to-left-1/2',
        'data-[state=closed]:slide-out-to-top-[48%]',
        'data-[state=open]:slide-in-from-left-1/2',
        'data-[state=open]:slide-in-from-top-[48%]',
        'z-50',
        'max-h-[90vh] overflow-y-auto',
        className
      )}
      {...props}
    >
      {children}
      <Dialog.Close asChild>
        <button
          className={cn(
            'absolute right-4 top-4 p-2 text-gray-400',
            'hover:text-gray-600 dark:hover:text-gray-300',
            'rounded-full hover:bg-gray-100 dark:hover:bg-gray-800',
            'transition-all duration-200'
          )}
          aria-label="关闭"
        >
          <X className="h-4 w-4" />
        </button>
      </Dialog.Close>
    </Dialog.Content>
  )
);

BaseDialogContent.displayName = 'BaseDialogContent';

const BaseDialogHeader: React.FC<BaseDialogHeaderProps> = ({ children, className, ...props }) => (
  <div className={cn('p-6 pb-4', className)} {...props}>
    {children}
  </div>
);

const BaseDialogFooter: React.FC<BaseDialogFooterProps> = ({ children, className, ...props }) => (
  <div className={cn('flex gap-3 p-6 pt-0 justify-end', className)} {...props}>
    {children}
  </div>
);

const BaseDialogTitle = React.forwardRef<HTMLHeadingElement, BaseDialogTitleProps>(
  ({ children, className, ...props }, ref) => (
    <Dialog.Title
      ref={ref}
      className={cn('text-lg font-semibold text-gray-900 dark:text-white', className)}
      {...props}
    >
      {children}
    </Dialog.Title>
  )
);

BaseDialogTitle.displayName = 'BaseDialogTitle';

const BaseDialogDescription = React.forwardRef<HTMLParagraphElement, BaseDialogDescriptionProps>(
  ({ children, className, ...props }, ref) => (
    <Dialog.Description
      ref={ref}
      className={cn('text-sm text-gray-600 dark:text-gray-400', className)}
      {...props}
    >
      {children}
    </Dialog.Description>
  )
);

BaseDialogDescription.displayName = 'BaseDialogDescription';

export {
  BaseDialog,
  BaseDialogContent,
  BaseDialogHeader,
  BaseDialogFooter,
  BaseDialogTitle,
  BaseDialogDescription,
};
