Feature: Config feature

  Scenario: An user with admin role can set the config
    Given a meta names contract
    And Alice user with the admin role
    When Alice updates the config 'whitelist_enabled' to 'true'
    Then the contract config 'whitelist_enabled' is 'true'

  Scenario: A user with no role cannot set the config
    Given a meta names contract
    When Alice updates the config 'whitelist_enabled' to 'true'
    Then the contract config 'whitelist_enabled' is 'false'

  Scenario: A user with whitelist role cannot set the config
    Given a meta names contract
    And Alice user with the whitelist role
    When Alice updates the config 'whitelist_enabled' to 'true'
    Then the contract config 'whitelist_enabled' is 'false'
