import { universalInvoke } from '@/lib/invoke-adapter';
import { AccountMetrics } from '@/commands/types/account.types.ts';

export class AccountMetricsCommands {
    /**
     * 获取账户配额指标 (Rust Backend Orchestrated - Singular)
     * @param email 账户邮箱
     */
    static async getAccountMetrics(email: string): Promise<AccountMetrics> {
        return universalInvoke('get_account_metrics', { email });
    }
}
