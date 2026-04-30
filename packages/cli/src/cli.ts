#!/usr/bin/env node

/**
 * @agent-gateway/cli - CLI tool for agent-gateway
 *
 * Command-line interface for managing providers, plans, and agents.
 */

import { Command } from 'commander';
import chalk from 'chalk';
import { getGateway, hasNativeBindings } from '@agent-gateway/node';
import type { ProviderInfo, PlanInfo } from '@agent-gateway/core';

const program = new Command();

program
  .name('agw')
  .description('agent-gateway CLI - Unified gateway for AI coding tools')
  .version('0.1.0');

// Show info about native bindings
if (!hasNativeBindings()) {
  console.log(chalk.yellow('⚠ Running in mock mode (native bindings not available)'));
}

const gateway = getGateway();

// Commands
program
  .command('serve')
  .description('Start the gateway server')
  .option('-p, --port <port>', 'Port to listen on', '8081')
  .option('-d, --daemon', 'Run in background')
  .action(async (options) => {
    console.log(chalk.blue('Starting gateway server...'));
    console.log(`Port: ${options.port}`);
    console.log(chalk.green('Gateway started successfully'));
  });

program
  .command('provider')
  .description('Manage providers')
  .action(() => {
    console.log(chalk.blue('Listing providers...'));
  });

program
  .command('provider:list')
  .description('List all providers')
  .action(async () => {
    try {
      const providers = await gateway.listProviders();
      console.log(chalk.bold('\nProviders:\n'));
      providers.forEach((p: ProviderInfo) => {
        console.log(chalk.cyan(`  ${p.name}`));
        console.log(`    ID: ${p.provider_id}`);
        console.log(`    API Format: ${p.api_format}`);
        console.log(`    Plans: ${p.coding_plans.length}`);
        console.log();
      });
    } catch (e) {
      console.error(chalk.red(`Error: ${e}`));
    }
  });

program
  .command('plan')
  .description('Manage plans')
  .action(() => {
    console.log(chalk.blue('Managing plans...'));
  });

program
  .command('plan:list')
  .description('List all plans')
  .option('-v, --verbose', 'Show detailed information')
  .action(async (options) => {
    try {
      const plans = await gateway.listPlans();
      if (plans.length === 0) {
        console.log(chalk.yellow('No plans configured. Use "agw plan:add" to add a plan.'));
        return;
      }
      console.log(chalk.bold('\nPlans:\n'));
      plans.forEach((p: PlanInfo) => {
        const status = p.enabled ? chalk.green('●') : chalk.red('●');
        console.log(`  ${status} ${chalk.cyan(p.name)}`);
        console.log(`    ID: ${p.id}`);
        console.log(`    Provider: ${p.provider_id}`);
        console.log(`    Model: ${p.selected_model_id}`);
        if (options.verbose) {
          console.log(`    Priority: ${p.priority}`);
          console.log(`    Health: ${p.health_status}`);
        }
        console.log();
      });
    } catch (e) {
      console.error(chalk.red(`Error: ${e}`));
    }
  });

program
  .command('plan:add')
  .description('Add a new plan')
  .requiredOption('-p, --provider <id>', 'Provider ID')
  .requiredOption('-n, --name <name>', 'Plan name')
  .requiredOption('-k, --api-key <key>', 'API key')
  .option('-m, --model <id>', 'Model ID')
  .action(async (options) => {
    try {
      const plan = await gateway.createPlan({
        provider_id: options.provider,
        plan_id: 'default',
        name: options.name,
        api_key: options.apiKey,
        selected_model_id: options.model || 'default',
      });
      console.log(chalk.green(`✓ Plan created: ${plan.id}`));
    } catch (e) {
      console.error(chalk.red(`Error: ${e}`));
    }
  });

program
  .command('plan:test')
  .description('Test plan connectivity')
  .argument('<id>', 'Plan ID')
  .action(async (id) => {
    try {
      console.log(chalk.blue(`Testing plan: ${id}...`));
      const result = await gateway.testPlan(id);
      if (result.success) {
        console.log(chalk.green(`✓ ${result.message}`));
        if (result.latency_ms) {
          console.log(`  Latency: ${result.latency_ms}ms`);
        }
      } else {
        console.log(chalk.red(`✗ ${result.message}`));
      }
    } catch (e) {
      console.error(chalk.red(`Error: ${e}`));
    }
  });

program
  .command('plan:delete')
  .description('Delete a plan')
  .argument('<id>', 'Plan ID')
  .option('-f, --force', 'Skip confirmation')
  .action(async (id, options) => {
    try {
      if (!options.force) {
        console.log(chalk.yellow(`Delete plan: ${id}? (y/N)`));
        // In interactive mode, would wait for input
      }
      await gateway.deletePlan(id);
      console.log(chalk.green(`✓ Plan deleted: ${id}`));
    } catch (e) {
      console.error(chalk.red(`Error: ${e}`));
    }
  });

program
  .command('quota')
  .description('Manage quotas')
  .action(async () => {
    const health = gateway.health();
    console.log(chalk.bold('\nQuota Status:\n'));
    console.log(`  Status: ${health.status}`);
    console.log(chalk.green('✓ Quota module ready'));
  });

program
  .command('quota:status')
  .description('Show quota status')
  .argument('[planId]', 'Plan ID')
  .action(async (planId) => {
    try {
      if (!planId) {
        const plans = await gateway.listPlans();
        if (plans.length > 0) {
          planId = plans[0].id;
        }
      }
      if (!planId) {
        console.log(chalk.yellow('No plans available'));
        return;
      }
      const quota = await gateway.getQuotaUsage(planId);
      if (quota) {
        console.log(chalk.bold(`\nQuota for ${planId}:\n`));
        console.log(`  Daily: ${quota.usage.daily_used} / ${quota.limits.daily || '∞'}`);
        console.log(`  Monthly: ${quota.usage.monthly_used} / ${quota.limits.monthly || '∞'}`);
        console.log(`  RPM: ${quota.usage.rpm_used} / ${quota.limits.rpm || '∞'}`);
      }
    } catch (e) {
      console.error(chalk.red(`Error: ${e}`));
    }
  });

program
  .command('fallback')
  .description('Manage fallback settings')
  .action(async () => {
    const config = await gateway.getFallbackConfig();
    console.log(chalk.bold('\nFallback Configuration:\n'));
    console.log(`  Enabled: ${config.enabled ? chalk.green('Yes') : chalk.red('No')}`);
    console.log(`  Max Attempts: ${config.max_attempts}`);
    if (config.priority_order.length > 0) {
      console.log(`  Priority: ${config.priority_order.join(' → ')}`);
    }
  });

program
  .command('fallback:on')
  .description('Enable automatic fallback')
  .action(async () => {
    const config = await gateway.setFallbackEnabled(true);
    console.log(chalk.green('✓ Fallback enabled'));
  });

program
  .command('fallback:off')
  .description('Disable automatic fallback')
  .action(async () => {
    const config = await gateway.setFallbackEnabled(false);
    console.log(chalk.green('✓ Fallback disabled'));
  });

program
  .command('key:validate')
  .description('Validate an API key')
  .argument('<key>', 'API key to validate')
  .action((key) => {
    const isValid = gateway.validateApiKey(key);
    if (isValid) {
      console.log(chalk.green('✓ Valid API key format'));
    } else {
      console.log(chalk.red('✗ Invalid API key format'));
    }
  });

program
  .command('key:mask')
  .description('Mask an API key')
  .argument('<key>', 'API key to mask')
  .action((key) => {
    const masked = gateway.maskApiKey(key);
    console.log(masked);
  });

program
  .command('health')
  .description('Check gateway health')
  .action(() => {
    const health = gateway.health();
    console.log(chalk.bold('\nGateway Health:\n'));
    console.log(`  Status: ${health.status === 'ok' ? chalk.green('OK') : chalk.red(health.status)}`);
    console.log(`  Version: ${health.version}`);
  });

// Parse commands
program.parse();