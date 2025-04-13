import axios, {type AxiosInstance} from 'axios';


export const createBaseURL = function (port: string, protocol?: string): string {
    if (typeof window !== 'undefined' && window.location?.port != "") {
        const location: any = window.location;

        return (protocol || location.protocol) + '//'+location.hostname+':' + port
    } else {
        return 'http://localhost:' + port
    }
}

interface TokenResponse {
    token: string;
}

interface UserResponse {
    id: number;
    username: string;
}

export class AccountService {
    private axiosClient: AxiosInstance;

    constructor(baseURL?: string) {
        if (!baseURL) {
            baseURL = createBaseURL("8080")
        }

        this.axiosClient = axios.create({
            baseURL: baseURL,
            headers: {'Content-Type': 'application/json'},
        })
    }
    async register(username: string, password: string) {
        const response = await this.axiosClient.post<TokenResponse>("/register", {name: username, password: password});
        return response.data;
    }

    async login(username: string, password: string) {
        const response = await this.axiosClient.post<TokenResponse>("/login", {name: username, password: password});
        return response.data;
    }

    async me(token: string) { // TODO auth token middleware
        const response = await this.axiosClient.get<UserResponse>("/me", {
            headers: {
                Authorization: `Bearer ${token}`,
                'Content-Type': 'application/json',
            }
        });
        return response.data;
    }
}

export const accountService = new AccountService(
    "https://roulette.botenza.org/api/account" // TODO parameterize baseURL
);