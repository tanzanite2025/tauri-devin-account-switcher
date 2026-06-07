pub fn get_injection_script(account_id: &str, email: &str, password: &str) -> String {
    format!(
        r#"
        (function() {{
            const accountId = "{}";
            const email = "{}";
            const password = "{}";
            
            // 1. 代理拦截 window.fetch 获取 JWT Token
            const orgFetch = window.fetch;
            window.fetch = async function(...args) {{
                const reqOrUrl = args[0];
                const urlString = typeof reqOrUrl === 'string' ? reqOrUrl : (reqOrUrl && reqOrUrl.url ? reqOrUrl.url : "");
                const options = args[1] || {{}};
                
                let headersObj = options.headers;
                if (!headersObj && reqOrUrl && reqOrUrl.headers) {{
                    headersObj = reqOrUrl.headers;
                }}
                
                if (headersObj) {{
                    let authHeader = "";
                    if (headersObj instanceof Headers) {{
                        authHeader = headersObj.get("Authorization") || "";
                    }} else if (typeof headersObj === "object") {{
                        authHeader = headersObj["Authorization"] || headersObj["authorization"] || "";
                    }}
                    if (authHeader && authHeader.includes("devin-session-token$")) {{
                        const tokenVal = authHeader.replace("Bearer ", "").trim();
                        if (window.__TAURI__ && window.__TAURI__.core.invoke) {{
                            window.__TAURI__.core.invoke("bind_captured_token", {{ id: accountId, token: tokenVal }})
                                .catch(err => console.error(err));
                        }}
                    }}
                }}
                
                const res = await orgFetch(...args);
                
                try {{
                    const clone = res.clone();
                    clone.json().then(data => {{
                        if (urlString.includes('/billing/status')) {{
                            if (window.__TAURI__ && window.__TAURI__.core.invoke) {{
                                window.__TAURI__.core.invoke("update_account_quota", {{
                                    id: accountId,
                                    billingError: data.billing_error || null,
                                    availableCredits: data.available_credits !== undefined ? data.available_credits : null
                                }}).catch(err => console.error("[CRITICAL] Failed to invoke update_account_quota", err));
                            }}
                        }}

                        let foundToken = null;
                        
                        function search(obj) {{
                            if (!obj || foundToken) return;
                            if (typeof obj === "string") {{
                                if (obj.includes("devin-session-token$")) {{
                                    foundToken = obj;
                                }} else if (obj.startsWith("ey") && obj.length > 50) {{
                                    foundToken = "devin-session-token$" + obj;
                                }}
                                return;
                            }}
                            if (typeof obj === "object") {{
                                for (const k in obj) {{
                                    if (k === "token" || k === "accessToken" || k === "sessionToken" || k === "jwt") {{
                                        const val = obj[k];
                                        if (typeof val === "string") {{
                                            if (val.includes("devin-session-token$")) {{
                                                foundToken = val;
                                            }} else if (val.startsWith("ey") && val.length > 50) {{
                                                foundToken = "devin-session-token$" + val;
                                            }}
                                        }}
                                    }}
                                    search(obj[k]);
                                }}
                            }}
                        }}
                        
                        search(data);
                        if (foundToken) {{
                            if (window.__TAURI__ && window.__TAURI__.core.invoke) {{
                                window.__TAURI__.core.invoke("bind_captured_token", {{ id: accountId, token: foundToken }})
                                    .catch(err => console.error(err));
                            }}
                        }}
                    }}).catch((err) => {{ console.error("[CRITICAL] Failed to parse JSON or invoke bind_captured_token", err); }});
                }} catch(e) {{ console.error("[CRITICAL] Intercept fetch failed", e); }}
                
                return res;
            }};

            // 2. 自动填单与登录提交
            let submitted = false;
            function fillAndSubmit() {{
                if (submitted) return false;
                
                let emailFilled = false;
                let passFilled = false;
                
                const emailFields = document.querySelectorAll('input[type="email"], input[name="email"], input[name="username"], input[id*="email"], input[id*="username"]');
                const passFields = document.querySelectorAll('input[type="password"], input[name="password"], input[id*="password"]');
                
                if (emailFields.length > 0 && email) {{
                    for (const f of emailFields) {{
                        if (f.value !== email) {{
                            f.value = email;
                            f.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            f.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        }}
                    }}
                    emailFilled = true;
                }} else if (!email) {{
                    emailFilled = true;
                }}
                
                if (passFields.length > 0 && password) {{
                    for (const f of passFields) {{
                        if (f.value !== password) {{
                            f.value = password;
                            f.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            f.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        }}
                    }}
                    passFilled = true;
                }} else if (!password) {{
                    passFilled = true;
                }}
                
                if (emailFilled && passFilled) {{
                    const submitBtn = document.querySelector('button[type="submit"], button.login-btn, input[type="submit"], button[class*="login"], button[id*="login"]');
                    if (submitBtn) {{
                        submitted = true;
                        submitBtn.click();
                        clearInterval(timer);
                        return true;
                    }}
                }}
                return false;
            }}
            
            const timer = setInterval(fillAndSubmit, 500);
            setTimeout(() => clearInterval(timer), 20000);

            // 3. 保持 Plan 自动刷新
            async function checkPlan() {{
                try {{
                    const res = await fetch('/api/auth/session');
                    if (res.ok) {{
                        const data = await res.json();
                        let plan = null;
                        let orgId = null;
                        if (data) {{
                            if (data.plan) plan = data.plan;
                            else if (data.user && data.user.plan) plan = data.user.plan;
                            else if (data.tier) plan = data.tier;
                            else if (data.user && data.user.tier) plan = data.user.tier;
                            
                            if (data.user && data.user.org_id) orgId = data.user.org_id;
                            else if (data.user && Array.isArray(data.user.organizations) && data.user.organizations.length > 0) orgId = data.user.organizations[0].id;
                            else if (data.organizations && Array.isArray(data.organizations) && data.organizations.length > 0) orgId = data.organizations[0].id;
                            else if (data.orgs && Array.isArray(data.orgs) && data.orgs.length > 0) orgId = data.orgs[0].id;
                        }}
                        
                        if (plan) {{
                            const upperPlan = plan.toUpperCase();
                            if (window.__TAURI__ && window.__TAURI__.core.invoke) {{
                                await window.__TAURI__.core.invoke("update_account_plan", {{ id: accountId, plan: upperPlan }});
                            }}
                        }}
                        
                        if (orgId) {{
                            try {{
                                const billingRes = await fetch(`/api/${{orgId}}/billing/status`);
                                if (billingRes.ok) {{
                                    const billingData = await billingRes.json();
                                    if (window.__TAURI__ && window.__TAURI__.core.invoke) {{
                                        await window.__TAURI__.core.invoke("update_account_quota", {{
                                            id: accountId,
                                            billingError: billingData.billing_error || null,
                                            availableCredits: billingData.available_credits !== undefined ? billingData.available_credits : null
                                        }}).catch(err => console.error("[CRITICAL] Failed to invoke update_account_quota", err));
                                    }}
                                }}
                            }} catch (err) {{
                                console.error("[CRITICAL] Failed to fetch billing status", err);
                            }}
                        }}
                        return true;
                    }}
                }} catch (e) {{ console.error("[CRITICAL] fetch session plan failed", e); }}
                return false;
            }}

            let checkCount = 0;
            const planTimer = setInterval(async () => {{
                checkCount++;
                const success = await checkPlan();
                if (success || checkCount > 10) {{
                    clearInterval(planTimer);
                }}
            }}, 5000);
        }})();
        "#,
        account_id.replace('"', "\\\""),
        email.replace('"', "\\\""),
        password.replace('"', "\\\"")
    )
}
