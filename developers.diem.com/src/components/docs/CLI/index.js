import React from 'react';
import classnames from 'classnames';
import PropTypes from 'prop-types';
import Terminal, {
  CommandForms as DefaultCommands,
} from '@libra-opensource/diem-cli';

import utils from 'diem-docusaurus-components/src/utils';

import commandsStyles from './css/commands.module.css';
import generalStyles from './css/general.module.css';
import tutorialStyles from './css/tutorial.module.css';

const {mergeCSSModules} = utils;

const styles = mergeCSSModules(
  {},
  commandsStyles,
  generalStyles,
  tutorialStyles
);

const getAvailableCommands = (command, commands) => {
  if (command && commands) {
    throw 'command and commands can not be simultaneously set on a CLI';
  } else if (!commands && !command) {
    return DefaultCommands;
  } else if (command) {
    return [command];
  } else {
    return commands;
  }
};

const CLI = ({ command, commands, isEmbedded, withTutorial }) => {
  const availableCommands = getAvailableCommands(command, commands);

  return (
    <Terminal
      availableCommands={availableCommands}
      className={classnames(styles.root, {
        [styles['embedded']]: isEmbedded,
        [styles['with-tutorial']]: withTutorial,
      })}
      withTutorial={withTutorial}
    />
  );
};

CLI.propTypes = {
  /*
   * Additional prop to simplify usage when there
   * is only one command
   */
  command: PropTypes.object,
  commands: PropTypes.arrayOf(PropTypes.object),
  isEmbedded: PropTypes.bool,
  withTutorial: PropTypes.bool,
};

CLI.defaultProps = {
  isEmbedded: true,
};

export default CLI;
