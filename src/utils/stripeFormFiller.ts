import { VirtualCard } from './cardGenerator';

/**
 * Stripe表单填充器类
 */
export class StripeFormFiller {
  private card: VirtualCard;

  constructor(card: VirtualCard) {
    this.card = card;
  }

  /**
   * 开始填充表单
   */
  async fillForm(): Promise<void> {
    console.log('开始自动填写Stripe支付表单...');
    
    // 等待页面加载
    await this.waitForPageLoad();
    
    // 填写各个字段
    await this.fillCardNumber();
    await this.fillExpiry();
    await this.fillCVC();
    await this.fillBillingInfo();
    
    console.log('表单填写完成！');
  }

  /**
   * 等待页面加载完成
   */
  private async waitForPageLoad(): Promise<void> {
    return new Promise((resolve) => {
      if (document.readyState === 'complete') {
        setTimeout(resolve, 1000); // 额外等待1秒确保表单渲染完成
      } else {
        window.addEventListener('load', () => {
          setTimeout(resolve, 1000);
        });
      }
    });
  }

  /**
   * 等待元素出现
   */
  private async waitForElement(selector: string, timeout: number = 10000): Promise<Element | null> {
    const startTime = Date.now();
    
    return new Promise((resolve) => {
      const checkElement = () => {
        const element = document.querySelector(selector);
        
        if (element) {
          resolve(element);
        } else if (Date.now() - startTime > timeout) {
          console.warn(`元素 ${selector} 未找到，超时`);
          resolve(null);
        } else {
          setTimeout(checkElement, 100);
        }
      };
      
      checkElement();
    });
  }

  /**
   * 模拟用户输入
   */
  private simulateInput(element: HTMLInputElement, value: string): void {
    // 聚焦元素
    element.focus();
    
    // 清空现有值
    element.value = '';
    
    // 逐字符输入以模拟真实用户行为
    for (let i = 0; i < value.length; i++) {
      const char = value[i];
      element.value += char;
      
      // 触发各种事件
      const keydownEvent = new KeyboardEvent('keydown', {
        key: char,
        bubbles: true,
        cancelable: true
      });
      element.dispatchEvent(keydownEvent);
      
      const inputEvent = new InputEvent('input', {
        data: char,
        bubbles: true,
        cancelable: true
      });
      element.dispatchEvent(inputEvent);
      
      const keyupEvent = new KeyboardEvent('keyup', {
        key: char,
        bubbles: true,
        cancelable: true
      });
      element.dispatchEvent(keyupEvent);
    }
    
    // 触发change和blur事件
    element.dispatchEvent(new Event('change', { bubbles: true }));
    element.dispatchEvent(new Event('blur', { bubbles: true }));
  }

  /**
   * 填写卡号
   */
  private async fillCardNumber(): Promise<void> {
    const element = await this.waitForElement('#cardNumber') as HTMLInputElement;
    if (element) {
      // 移除空格，Stripe表单会自动格式化
      const cardNumber = this.card.card_number.replace(/\s/g, '');
      this.simulateInput(element, cardNumber);
      console.log('✓ 已填写卡号');
    }
  }

  /**
   * 填写有效期
   */
  private async fillExpiry(): Promise<void> {
    const element = await this.waitForElement('#cardExpiry') as HTMLInputElement;
    if (element) {
      this.simulateInput(element, this.card.expiry_date);
      console.log('✓ 已填写有效期');
    }
  }

  /**
   * 填写CVC
   */
  private async fillCVC(): Promise<void> {
    const element = await this.waitForElement('#cardCvc') as HTMLInputElement;
    if (element) {
      this.simulateInput(element, this.card.cvv);
      console.log('✓ 已填写CVC');
    }
  }

  /**
   * 填写账单信息
   */
  private async fillBillingInfo(): Promise<void> {
    // 填写持卡人姓名
    const nameElement = await this.waitForElement('#billingName') as HTMLInputElement;
    if (nameElement) {
      this.simulateInput(nameElement, this.card.cardholder_name);
      console.log('✓ 已填写持卡人姓名');
    }
    
    // 选择国家（美国）
    const countryElement = await this.waitForElement('#billingCountry') as HTMLSelectElement;
    if (countryElement) {
      countryElement.value = this.card.billing_address.country;
      countryElement.dispatchEvent(new Event('change', { bubbles: true }));
      console.log('✓ 已选择国家');
      
      // 等待州选项加载
      await this.delay(500);
      
      // 选择州
      const stateElement = await this.waitForElement('#billingAdministrativeArea') as HTMLSelectElement;
      if (stateElement) {
        // 查找匹配的州选项
        const options = Array.from(stateElement.options);
        const matchingOption = options.find(opt => 
          opt.value === this.card.billing_address.state || 
          opt.text.includes(this.card.billing_address.state)
        );
        
        if (matchingOption) {
          stateElement.value = matchingOption.value;
          stateElement.dispatchEvent(new Event('change', { bubbles: true }));
          console.log('✓ 已选择州');
        }
      }
    }
    
    // 填写邮编
    const postalElement = await this.waitForElement('#billingPostalCode') as HTMLInputElement;
    if (postalElement) {
      this.simulateInput(postalElement, this.card.billing_address.postal_code);
      console.log('✓ 已填写邮编');
    }
    
    // 填写城市
    const cityElement = await this.waitForElement('#billingLocality') as HTMLInputElement;
    if (cityElement) {
      this.simulateInput(cityElement, this.card.billing_address.city);
      console.log('✓ 已填写城市');
    }
    
    // 填写地址
    const addressElement = await this.waitForElement('#billingAddressLine1') as HTMLInputElement;
    if (addressElement) {
      this.simulateInput(addressElement, this.card.billing_address.street_address);
      console.log('✓ 已填写地址');
    }
  }

  /**
   * 延迟函数
   */
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * 高亮提交按钮（可选）
   */
  async highlightSubmitButton(): Promise<void> {
    const submitButton = await this.waitForElement('button[type="submit"]') as HTMLButtonElement;
    if (submitButton) {
      // 添加脉冲动画效果
      submitButton.style.animation = 'pulse 2s infinite';
      
      // 添加CSS动画
      const style = document.createElement('style');
      style.innerHTML = `
        @keyframes pulse {
          0% { box-shadow: 0 0 0 0 rgba(99, 217, 203, 0.7); }
          70% { box-shadow: 0 0 0 10px rgba(99, 217, 203, 0); }
          100% { box-shadow: 0 0 0 0 rgba(99, 217, 203, 0); }
        }
      `;
      document.head.appendChild(style);
      
      console.log('✨ 表单已准备就绪，可以点击提交按钮');
    }
  }

  /**
   * 等待提交按钮变为可点击状态
   */
  private async waitForSubmitButtonReady(timeout: number = 30000): Promise<HTMLButtonElement | null> {
    const startTime = Date.now();
    
    return new Promise((resolve) => {
      const checkButton = () => {
        // 查找提交按钮
        const submitButton = document.querySelector('button[type="submit"]') as HTMLButtonElement;
        
        if (submitButton) {
          // 检查按钮是否包含 complete 类名
          const isComplete = submitButton.classList.contains('SubmitButton--complete');
          // 检查按钮文字是否为"开始试用"
          const buttonText = submitButton.querySelector('.SubmitButton-Text--current')?.textContent;
          const isStartTrial = buttonText?.includes('开始试用') || buttonText?.includes('Start trial');
          
          if (isComplete && isStartTrial && !submitButton.disabled) {
            console.log('✅ 提交按钮已就绪：', buttonText);
            resolve(submitButton);
            return;
          } else if (isComplete && !submitButton.disabled) {
            // 如果按钮已经是complete状态但文字不同，也可以点击
            console.log('✅ 提交按钮已就绪（complete状态）');
            resolve(submitButton);
            return;
          } else {
            // 输出当前状态用于调试
            if (!isComplete) {
              console.log('⏳ 等待按钮变为可点击状态...');
            }
          }
        }
        
        // 检查是否超时
        if (Date.now() - startTime > timeout) {
          console.error('❌ 等待提交按钮超时');
          resolve(null);
        } else {
          setTimeout(checkButton, 500);
        }
      };
      
      checkButton();
    });
  }

  /**
   * 自动提交表单（谨慎使用）
   */
  async autoSubmit(delay: number = 3000): Promise<void> {
    console.log(`将在 ${delay/1000} 秒后检查提交按钮...`);
    await this.delay(delay);
    
    console.log('🔍 正在等待提交按钮变为可点击状态...');
    const submitButton = await this.waitForSubmitButtonReady();
    
    if (submitButton) {
      // 滚动到按钮位置
      submitButton.scrollIntoView({ behavior: 'smooth', block: 'center' });
      await this.delay(500);
      
      // 点击提交按钮
      submitButton.click();
      console.log('✅ 已自动点击提交按钮');
      
      // 再次点击以确保提交（有时需要）
      setTimeout(() => {
        if (submitButton && !submitButton.disabled) {
          submitButton.click();
          console.log('✅ 再次确认点击提交按钮');
        }
      }, 1000);
    } else {
      console.error('❌ 未找到可用的提交按钮或等待超时');
    }
  }
}

/**
 * 创建并执行表单填充器
 */
export async function autoFillStripeForm(card: VirtualCard, autoSubmit: boolean = false): Promise<void> {
  try {
    const filler = new StripeFormFiller(card);
    await filler.fillForm();
    await filler.highlightSubmitButton();
    
    if (autoSubmit) {
      await filler.autoSubmit();
    }
  } catch (error) {
    console.error('自动填写表单失败:', error);
  }
}

/**
 * 检查是否在Stripe支付页面
 */
export function isStripePaymentPage(): boolean {
  const url = window.location.href;
  return url.includes('checkout.stripe.com') || url.includes('stripe.com');
}
