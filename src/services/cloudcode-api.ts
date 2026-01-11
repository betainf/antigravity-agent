import {fetch} from '@tauri-apps/plugin-http'
import {CloudCodeAPITypes} from "@/services/cloudcode-api.types.ts";

// HTTP 客户端配置
interface HTTPConfig {
  baseURL: string;
  headers: Record<string, string>;
}

const HTTP_CONFIG: HTTPConfig = {
  baseURL: 'https://daily-cloudcode-pa.sandbox.googleapis.com', // 默认使用沙盒环境
  headers: {
    "User-Agent": "antigravity/windows/amd64",
    "Content-Type": "application/json",
    "Accept": "application/json"
  }
};


const post = async <T>(endpoint: string, data: any, options?: RequestInit): Promise<T> => {

  const requestConfig: RequestInit = {
    method: 'POST',
    body: JSON.stringify(data),
    ...options,
    headers: {
      ...HTTP_CONFIG.headers,
      ...(options?.headers || {})
    }
  };

  const response = await fetch(`${HTTP_CONFIG.baseURL}${endpoint}`, requestConfig);

  return await response.json();
}


// CloudCode API 服务命名空间
export namespace CloudCodeAPI {

  export async function fetchAvailableModels(
    authorizationKey: string,
    project: string,
  ): Promise<CloudCodeAPITypes.FetchAvailableModelsResponse> {

    const requestData = {
      "project": project
    };

    const response = await post<CloudCodeAPITypes.FetchAvailableModelsResponse | CloudCodeAPITypes.ErrorResponse>(
      '/v1internal:fetchAvailableModels',
      requestData,
      {
        headers: {
          'Authorization': `Bearer ${authorizationKey}`
        }
      }
    );

    if ("error" in response) {
      return Promise.reject(response);
    }

    return response;
  }

  export async function loadCodeAssist(
    authorizationKey: string,
  ) {
    const requestData = {metadata: {ideType: 'ANTIGRAVITY'}};

    const response = await post<CloudCodeAPITypes.LoadCodeAssistResponse | CloudCodeAPITypes.ErrorResponse>(
      '/v1internal:loadCodeAssist',
      requestData,
      {
        headers: {
          'Authorization': `Bearer ${authorizationKey}`
        }
      }
    )

    if ("error" in response) {
      return Promise.reject(response);
    }

    return response;
  }

  /**
   * token 刷新应在后端完成，避免将 client_secret 暴露到前端
   */

  export async function userinfo(
    access_token: string,
  ) {

    const response = await fetch(
      'https://www.googleapis.com/oauth2/v2/userinfo',
      {
        headers: {
          'Authorization': `Bearer ${access_token}`
        }
      },
    );
    const json = await response.json() as unknown as CloudCodeAPITypes.UserInfoResponse | CloudCodeAPITypes.ErrorResponse;

    if ("error" in json) {
      return Promise.reject(json);
    }

    return json;
  }

}
