# 检查配置文件变动的时间间隔
refresh_rate: 10 seconds
# appender 负责将日志收集到控制台或文件, 可配置多个
appenders:

  stdout:
    kind: console
    encoder:
      pattern: "{d} [{P}] {h({l:>5.5})} {t} - {m}{n}"

  file:
    kind: file
    path: "log/bmsg.log"
    encoder:
      # log 信息模式
      pattern: "{d} [{P}] {h({l:>5.5})} {t} - {m}{n}"

  rfile:
    kind: rolling_file
    path: "log/bmsg.log"
    # Specifies if the appender should append to or truncate the log file if it
    # already exists. Defaults to `true`.
    append: true
    # The encoder to use to format output. Defaults to `kind: pattern`.
    encoder:
      pattern: "{d} [{P}] {h({l:>5.5})} {t} - {m}{n}"
    # The policy which handles rotation of the log file. Required.
    policy:
      # Identifies which policy is to be used. If no kind is specified, it will
      # default to "compound".
      kind: compound
      # The remainder of the configuration is passed along to the policy's
      # deserializer, and will vary based on the kind of policy.
      trigger:
        kind: size
        limit: 10 mb
      roller:
        kind: delete

# 对全局 log 进行配置
root:
  level: debug
  appenders:
    - stdout
    - rfile
