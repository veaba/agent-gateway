import { ref } from 'vue'
import type { UserPlan } from '@/types'
import { fetchPlans, createPlan, updatePlan, deletePlan, testPlan } from '@/api'

export function usePlans() {
  const plans = ref<UserPlan[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const loadPlans = async () => {
    isLoading.value = true
    error.value = null
    try {
      plans.value = await fetchPlans()
    } catch (e) {
      error.value = '加载套餐失败'
    } finally {
      isLoading.value = false
    }
  }

  const addPlan = async (plan: Partial<UserPlan>) => {
    try {
      const newPlan = await createPlan(plan)
      plans.value.push(newPlan)
      return newPlan
    } catch (e) {
      error.value = '创建套餐失败'
      throw e
    }
  }

  const editPlan = async (id: string, plan: Partial<UserPlan>) => {
    try {
      const updated = await updatePlan(id, plan)
      const index = plans.value.findIndex(p => p.id === id)
      if (index !== -1) {
        plans.value[index] = updated
      }
      return updated
    } catch (e) {
      error.value = '更新套餐失败'
      throw e
    }
  }

  const removePlan = async (id: string) => {
    try {
      await deletePlan(id)
      plans.value = plans.value.filter(p => p.id !== id)
    } catch (e) {
      error.value = '删除套餐失败'
      throw e
    }
  }

  const test = async (id: string) => {
    try {
      return await testPlan(id)
    } catch (e) {
      error.value = '测试连接失败'
      throw e
    }
  }

  return {
    plans,
    isLoading,
    error,
    loadPlans,
    addPlan,
    editPlan,
    removePlan,
    test
  }
}